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

use clokwerk::{Interval, TimeUnits};
use clokwerk::Interval::*;
use rori::database::Database;
use rori::interaction::Interaction;
use rori::user::Device;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

/**
 * Represents a task for the schedule
 * @note may be simplified in the future if we supports cron format
 */
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
    jobs: Arc<Mutex<HashMap<i32, clokwerk::Scheduler>>>,
    stop: Arc<AtomicBool>,
    thread: Option<thread::JoinHandle<()>>,
}

/**
 * When dropping a scheduler, all tasks are stopped and the thread should stop
 */
impl Drop for Scheduler {
    fn drop(&mut self) {
        self.jobs.lock().unwrap().clear();
        self.stop.store(true, Ordering::SeqCst);
        let _ = self.thread.take().unwrap().join();
    }
}

impl Scheduler {
    /**
     * Generate a new scheduler
     */
    pub fn new() -> Scheduler {
        let jobs = Arc::new(Mutex::new(HashMap::new()));
        let stop = Arc::new(AtomicBool::new(false));
        let cloned = jobs.clone();
        let stop_cloned = stop.clone();
        let thread = Some(thread::spawn(move|| {
            loop {
                if stop_cloned.load(Ordering::SeqCst) {
                    return;
                }
                let jobs: &mut HashMap<i32, clokwerk::Scheduler> = &mut cloned.lock().unwrap();
                for (_id, job) in jobs.into_iter() {
                    job.run_pending();
                }
                thread::sleep(Duration::from_secs(1));
            }
        }));
        let mut result = Scheduler {
            jobs,
            stop,
            thread,
        };

        result.load_tasks();
        result
    }

    /**
     * Add a task to the scheduler
     * @param self
     * @param content       Json representing the task
     * @return id of the task
     */
    pub fn add_task(&mut self, content: &String) -> Option<i32> {
        let content: HashMap<String, String> = serde_json::from_str(&*content).unwrap_or(HashMap::new());
        let task = ScheduledTask {
            id: content.get("id").unwrap().parse::<i32>().unwrap(),
            module: content.get("module").unwrap().parse::<i32>().unwrap(),
            parameter: content.get("parameter").unwrap().to_string(),
            at: content.get("at").unwrap().to_string(),
            seconds: content.get("seconds").unwrap().parse::<u32>().unwrap(),
            minutes: content.get("minutes").unwrap().parse::<u32>().unwrap(),
            hours: content.get("hours").unwrap().parse::<u32>().unwrap(),
            days: content.get("days").unwrap().to_string(),
            repeat: content.get("repeat").unwrap() == "True",
        };
        let result = Database::add_task(&task);
        if !result.is_none() {
            self.load_task(task);
        }
        result
    }

    /**
     * Update a task to the scheduler
     * @param self
     * @param content       Json representing the task
     * @return id of the task
     */
    pub fn update_task(&mut self, content: &String) -> Option<i32> {
        let content: HashMap<String, String> = serde_json::from_str(&*content).unwrap_or(HashMap::new());
        let task = ScheduledTask {
            id: content.get("id").unwrap().parse::<i32>().unwrap(),
            module: content.get("module").unwrap().parse::<i32>().unwrap(),
            parameter: content.get("parameter").unwrap().to_string(),
            at: content.get("at").unwrap().to_string(),
            seconds: content.get("seconds").unwrap().parse::<u32>().unwrap(),
            minutes: content.get("minutes").unwrap().parse::<u32>().unwrap(),
            hours: content.get("hours").unwrap().parse::<u32>().unwrap(),
            days: content.get("days").unwrap().to_string(),
            repeat: content.get("repeat").unwrap().parse::<bool>().unwrap(),
        };
        let result = Database::update_task(&task);
        if !result.is_ok() {
            self.load_task(task);
            return Some(result.unwrap() as i32)
        }
        None
    }

    /**
     * Remove a task from the scheduler
     * @param self
     * @param id       Id of the task
     * @return id of the task
     */
    pub fn rm_task(&mut self, id: &i32) -> Option<i32> {
        let result = Database::rm_task(&id);
        if result.is_ok() {
            let jobs: &mut HashMap<i32, clokwerk::Scheduler> = &mut self.jobs.lock().unwrap();
            jobs.retain(|jid, _| jid == id);
            return Some(result.unwrap() as i32)
        }
        None
    }

// private
    /**
     * Load tasks from the database
     * @param self
     */
    fn load_tasks(&mut self) {
        for task in Database::get_tasks() {
            self.load_task(task);
        }
    }

    /**
     * Load one task into the scheduler
     * @param self
     * @param task  The task to load
     */
    fn load_task(&mut self, task: ScheduledTask) {
        let mut scheduler = clokwerk::Scheduler::new();
        let mut interval: Option<Interval> = None;
        let mut first_interval = true;
        // May be simplified in the future via the cron format
        // Should be implemented in clokwerk
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

        // Load the module to run
        let module = Database::get_module(&task.module);

        if module.is_none() {
            warn!("Remove task with id {} because no module were found", task.id);
            let _ = Database::rm_task(&task.id);
            return;
        }
        let module = module.unwrap();
        let metadatas: HashMap<String, String> = serde_json::from_str(&*task.parameter).unwrap_or(HashMap::new());
        if metadatas.is_empty() {
            warn!("Remove task {} with id {} because no parameters were specified", module.name, task.id);
            let _ = Database::rm_task(&task.id);
            return;
        }

        let device = Database::get_device(&metadatas["ring_id"], &metadatas["username"]);
        if device.0 == -1 {
            warn!("Remove task {} with id {} because no device were found", module.name, task.id);
            let _ = Database::rm_task(&task.id);
            return;
        }
        let interaction = Interaction {
            device_author: Device::new(&device.0, &device.1),
            body: String::new(),
            metadatas: metadatas,
            datatype: String::new(),
            time: time::now()
        };

        info!("Scheduled new job for module {} with interaction {}", module.name, interaction);
        job.run(move || {
            info!("Scheduler exec job for module {} with interaction {}", module.name, interaction);
            module.exec(&interaction);
        });
        let jobs: &mut HashMap<i32, clokwerk::Scheduler> = &mut self.jobs.lock().unwrap();
        jobs.insert(task.id, scheduler);
    }
}