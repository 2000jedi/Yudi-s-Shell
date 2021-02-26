use subprocess::Popen;

pub struct Job {
    pub pid : u32,
    pub cmd : String,
    pub proc: Popen,
}

pub struct Jobs {
    pub jobs : Vec<Job>
}

impl Jobs {
    pub fn new() -> Jobs {
        return Jobs {jobs: Vec::new()};
    }

    pub fn print(&self) {
        let mut iter : i32 = 0;
        for job in &self.jobs {
            println!("[{}] ({}) {}", iter, job.pid, job.cmd);
            iter += 1;
        }
    }

    pub fn push(&mut self, proc : Popen, cmd : String) {
        let pid = match proc.pid() {
            Some(p) => p,
            None => {
                eprintln!("Cannot find PID of process");
                return;
            }
        };

        self.jobs.push(Job {pid, cmd, proc});
    }
}
