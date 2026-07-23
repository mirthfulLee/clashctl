use std::{
    sync::mpsc::{Receiver, Sender},
    thread::{JoinHandle, scope},
    time::Duration,
};

use clashctl_core::Clash;
use crossterm::event::Event as CrossTermEvent;
use log::{debug, warn};
use rayon::prelude::*;

use crate::{
    interactive::Flags,
    ui::{
        Action, TuiError, TuiOpt, TuiResult,
        event::{Event, UpdateEvent},
        utils::{Interval, Pulse},
    },
};

pub type Job = JoinHandle<TuiResult<()>>;

pub fn servo(tx: Sender<Event>, rx: Receiver<Action>, opt: TuiOpt, flags: Flags) -> TuiResult<()> {
    let clash = flags.connect_server_from_config()?;
    api_result("/version", clash.get_version())?;

    scope(|r| -> TuiResult<()> {
        let tx_clone = tx.clone();
        let error_tx = tx.clone();
        let handle1 = r.spawn(move || report_job_error(&error_tx, input_job(tx_clone)));

        let tx_clone = tx.clone();
        let error_tx = tx.clone();
        let clash_ref = &clash;
        let handle2 =
            r.spawn(move || report_job_error(&error_tx, traffic_job(tx_clone, clash_ref)));

        let tx_clone = tx.clone();
        let error_tx = tx.clone();
        let clash_ref = &clash;
        let handle3 = r.spawn(move || report_job_error(&error_tx, log_job(tx_clone, clash_ref)));

        let tx_clone = tx.clone();
        let error_tx = tx.clone();
        let clash_ref = &clash;
        let opt_ref = &opt;
        let flags_ref = &flags;
        let handle4 = r.spawn(move || {
            report_job_error(&error_tx, req_job(opt_ref, flags_ref, tx_clone, clash_ref))
        });

        let error_tx = tx.clone();
        let clash_ref = &clash;
        let opt_ref = &opt;
        let flags_ref = &flags;
        let handle5 = r.spawn(move || {
            report_job_error(&error_tx, action_job(opt_ref, flags_ref, tx, rx, clash_ref))
        });

        handle1.join().unwrap()?;
        handle2.join().unwrap()?;
        handle3.join().unwrap()?;
        handle4.join().unwrap()?;
        handle5.join().unwrap()?;

        Ok(())
    })
}

fn input_job(tx: Sender<Event>) -> TuiResult<()> {
    loop {
        match crossterm::event::read() {
            Ok(CrossTermEvent::Key(event)) => tx.send(Event::from(event))?,
            Err(_) => {
                tx.send(Event::Quit)?;
                break;
            }
            _ => {}
        }
    }
    Ok(())
}

fn req_job(_opt: &TuiOpt, _flags: &Flags, tx: Sender<Event>, clash: &Clash) -> TuiResult<()> {
    let mut interval = Interval::every(Duration::from_millis(50));
    let mut connection_pulse = Pulse::new(20); // Every 1 s
    let mut proxies_pulse = Pulse::new(100); //   Every 5 s + 0 tick
    let mut rules_pulse = Pulse::new(101); //     Every 5 s + 1 tick
    let mut version_pulse = Pulse::new(102); //   Every 5 s + 2 tick
    let mut config_pulse = Pulse::new(103); //    Every 5 s + 3 tick

    loop {
        if version_pulse.tick() {
            tx.send(Event::Update(UpdateEvent::Version(api_result(
                "/version",
                clash.get_version(),
            )?)))?;
        }
        if connection_pulse.tick() {
            tx.send(Event::Update(UpdateEvent::Connection(
                api_result("/connections", clash.get_connections())?.into(),
            )))?;
        }
        if rules_pulse.tick() {
            tx.send(Event::Update(UpdateEvent::Rules(api_result(
                "/rules",
                clash.get_rules(),
            )?)))?;
        }
        if proxies_pulse.tick() {
            tx.send(Event::Update(UpdateEvent::Proxies(api_result(
                "/proxies",
                clash.get_proxies(),
            )?)))?;
        }
        if config_pulse.tick() {
            tx.send(Event::Update(UpdateEvent::Config(api_result(
                "/configs",
                clash.get_configs(),
            )?)))?;
        }
        interval.tick();
    }
}

fn traffic_job(tx: Sender<Event>, clash: &Clash) -> TuiResult<()> {
    let mut traffics = api_result("/traffic", clash.get_traffic())?;
    loop {
        match traffics.next() {
            Some(Ok(traffic)) => tx.send(Event::Update(UpdateEvent::Traffic(traffic)))?,
            Some(Err(e)) => return Err(api_error("/traffic", e)),
            None => return Err(TuiError::ApiStreamEnded("/traffic")),
        }
    }
}

fn log_job(tx: Sender<Event>, clash: &Clash) -> TuiResult<()> {
    let mut logs = api_result("/logs", clash.get_log())?;
    loop {
        match logs.next() {
            Some(Ok(log)) => tx.send(Event::Update(UpdateEvent::Log(log)))?,
            Some(Err(e)) => return Err(api_error("/logs", e)),
            None => return Err(TuiError::ApiStreamEnded("/logs")),
        }
    }
}

fn action_job(
    _opt: &TuiOpt,
    flags: &Flags,
    tx: Sender<Event>,
    rx: Receiver<Action>,
    clash: &Clash,
) -> TuiResult<()> {
    while let Ok(action) = rx.recv() {
        tx.send(Event::Action(action.clone()))?;
        match action {
            Action::TestLatency { group, proxies } => {
                let result =
                    match clash.get_group_delay(&group, flags.test_url.as_str(), flags.timeout) {
                        Ok(_) => Vec::new(),
                        Err(error) => {
                            debug!(
                                "Group latency test unavailable ({error}); falling back to \
                                 individual proxies"
                            );
                            proxies
                                .par_iter()
                                .filter_map(|proxy| {
                                    clash
                                        .get_proxy_delay(
                                            proxy,
                                            flags.test_url.as_str(),
                                            flags.timeout,
                                        )
                                        .err()
                                })
                                .collect::<Vec<_>>()
                        }
                    };

                let count = result.len();

                if count != 0 {
                    warn!(
                        "   {}",
                        result
                            .into_iter()
                            .map(|x| x.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                    warn!("({}) error(s) during test proxy delay", count);
                }

                tx.send(Event::Update(UpdateEvent::ProxyTestLatencyDone))?;
                tx.send(Event::Update(UpdateEvent::Proxies(api_result(
                    "/proxies",
                    clash.get_proxies(),
                )?)))?;
            }
            Action::ApplySelection { group, proxy } => {
                api_result(
                    "/proxies/{group}",
                    clash.set_proxygroup_selected(&group, &proxy),
                )?;
                tx.send(Event::Update(UpdateEvent::Proxies(api_result(
                    "/proxies",
                    clash.get_proxies(),
                )?)))?;
            }
        }
    }
    Ok(())
}

fn api_result<T>(endpoint: &'static str, result: clashctl_core::Result<T>) -> TuiResult<T> {
    result.map_err(|source| api_error(endpoint, source))
}

fn api_error(endpoint: &'static str, source: clashctl_core::Error) -> TuiError {
    TuiError::ApiEndpoint { endpoint, source }
}

fn report_job_error(tx: &Sender<Event>, result: TuiResult<()>) -> TuiResult<()> {
    if let Err(error) = result {
        let _ = tx.send(Event::Failure(error.to_string()));
        return Err(error);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use super::*;

    #[test]
    fn background_errors_are_forwarded_to_the_ui() {
        let (tx, rx) = channel();

        let result = report_job_error(&tx, Err(TuiError::TuiBackendErr));

        assert!(result.is_err());
        match rx.recv().unwrap() {
            Event::Failure(message) => assert_eq!(message, "TUI backend error"),
            event => panic!("expected failure event, got {event:?}"),
        }
    }

    #[test]
    fn api_errors_include_the_endpoint() {
        let error = api_result::<()>(
            "/connections",
            Err(clashctl_core::Error::other("invalid payload".to_owned())),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Clash/Mihomo API `/connections` failed: Other errors (invalid payload)"
        );
    }
}
