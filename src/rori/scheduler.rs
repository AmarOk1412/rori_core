/**
 * Copyright (c) 2020, Sébastien Blin <sebastien.blin@enconn.fr>
 * All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are met:
 *
 * * Redistributions of source code must retain the above copyright
 *  notice, this list of conditions and the following disclaimer.
 * * Redistributions in binary form must reproduce the above copyright
 *  notice, this list of conditions and the following disclaimer in the
 *  documentation and/or other materials provided with the distribution.
 * * Neither the name of the University of California, Berkeley nor the
 *  names of its contributors may be used to endorse or promote products
 *  derived from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE REGENTS AND CONTRIBUTORS ``AS IS'' AND ANY
 * EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
 * DISCLAIMED. IN NO EVENT SHALL THE REGENTS AND CONTRIBUTORS BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES;
 * LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND
 * ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
 * (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
 * SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
 **/

use clokwerk::{Interval, Job, ScheduleHandle, TimeUnits};
use clokwerk::Interval::*;
use rori::database::Database;


use std::collections::HashMap;
use rori::interaction::Interaction;
use rori::user::{Device, User};
use rori::module::Module;
use rori::module::TextCondition;
use std::time::Duration;

pub struct ScheduledTask {
    pub id : i32,
    pub module : i32,
    pub parameter : String,
    pub at : String,
    pub seconds : u32,
    pub minutes : u32,
    pub hours : u32,
    pub days : String,
    pub repeat : bool
}

/**
 * The scheduler allows RORI to schedule the lauch of modules when needed
 */
pub struct Scheduler {
    handlers: Vec<ScheduleHandle>
}

impl Scheduler {
    pub fn new() -> Scheduler {
        let mut scheduler = clokwerk::Scheduler::new();
        scheduler.every(1.minutes()).run(|| {
            println!("@@@ Scheduler todo");
            let feed: Module = Module {
                condition: Box::new(TextCondition::new(String::new())),
                name: String::from("feed"),
                path: String::from("command/feed"),
                priority: 0,
                enabled: true,
            };
            let mut metadatas: HashMap<String, String> = HashMap::new();
            metadatas.insert(String::from("ch"), String::from("495695106758803456"));
            feed.exec(&Interaction {
                device_author: Device::new(&-1, &String::from("c2a2818dd78b95ec1bffb9845c421925970225c0")),
                body: String::new(),
                metadatas: metadatas,
                datatype: String::new(),
                time: time::now()
            });
        });

        let mut handlers = Vec::new();
        handlers.push(scheduler.watch_thread(Duration::from_secs(1)));
        let mut result = Scheduler {
            handlers,
        };

        result.load_tasks();
        result
    }

    fn load_tasks(&mut self) {
        let mut scheduler = clokwerk::Scheduler::new();
        for task in Database::get_tasks() {
            let mut interval: Option<Interval> = None;
            let mut first_interval = true;
            let mut job = scheduler.every(1.day());
            if task.days == "Monday" {
                interval = Some(Monday);
            } else if task.days == "Tuesday" {
                interval = Some(Tuesday);
            } else if task.days == "Wednesday" {
                interval = Some(Wednesday);
            } else if task.days == "Thursday" {
                interval = Some(Thursday);
            } else if task.days == "Friday" {
                interval = Some(Friday);
            } else if task.days == "Saturday" {
                interval = Some(Saturday);
            } else if task.days == "Sunday" {
                interval = Some(Sunday);
            } else if task.days == "Weekday" {
                interval = Some(Weekday);
            } else if !task.days.is_empty() {
                match task.days.parse::<u32>() {
                    Ok(d) => interval = Some(Days(d)),
                    _ => {}
                }
            }

            match interval {
                Some(interval) => {
                    first_interval = false;
                    job = scheduler.every(interval);
                },
                _ => {}
            }

            if task.hours != 0 {
                if first_interval {
                    first_interval = false;
                    job = scheduler.every(Hours(task.hours));
                } else {
                    job = job.plus(Hours(task.hours));
                }
            }

            if task.minutes != 0 {
                if first_interval {
                    first_interval = false;
                    job = scheduler.every(Minutes(task.minutes));
                } else {
                    job = job.plus(Minutes(task.minutes));
                }
            }

            if task.seconds != 0 {
                if first_interval {
                    first_interval = false;
                    job = scheduler.every(Seconds(task.seconds));
                } else {
                    job = job.plus(Seconds(task.seconds));
                }
            }

            if task.at != "" {
                if first_interval {
                    job = scheduler.every(Days(0));
                }
                job = job.at(&*task.at);
            }

            if !task.repeat {
                job = job.once();
            }

            let module = Database::get_module(&task.module);

            if module.is_none() {
                // TODO remove from db as malformed
                continue;
            }

            // TODO deserialize interaction?
            let metadatas: HashMap<String, String> = serde_json::from_str(&*task.parameter).unwrap_or(HashMap::new());

            if !metadatas.is_empty() {
                // TODO remove from db as malformed
                continue;
            }

            let devices = Database::get_devices_for_username(&metadatas["userc  "]);
            if devices.is_empty() {
                // TODO remove from db as malformed
                continue;
            }
            let interaction = Interaction {
                device_author: Device::new(&devices[0].0, &devices[0].1),
                body: String::new(),
                metadatas: metadatas,
                datatype: String::new(),
                time: time::now()
            };
            let module = module.unwrap();

            job.run(move || {
                info!("Scheduler exec job for module {} with interaction {}", module.name, interaction);
                module.exec(&interaction);
            });

        }
        // TODO only do one scheduler and run pending in another thread launched by manager
        self.handlers.push(scheduler.watch_thread(Duration::from_secs(1)));
    }
}