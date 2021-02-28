use subprocess::Popen;

pub struct Job {
    pub jid : u32,
    pub pid : u32,
    pub cmd : String,
    pub proc: Popen,
}

pub struct Jobs {
    pub jobs : Vec<Job>,
    pub cnt  : u32,
}

impl Jobs {
    pub fn new() -> Jobs {
        return Jobs {cnt: 1, jobs: Vec::new()};
    }

    pub fn print(&self) {
        for job in &self.jobs {
            println!("[{}] ({}) {}", job.jid, job.pid, job.cmd);
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

        self.jobs.push(Job {jid: self.cnt, pid, cmd, proc});
        self.cnt += 1;
    }

    pub fn refresh(&mut self) {
        let mut ind : usize = 0;
        let mut v : Vec<usize> = vec![];
        for job in &mut self.jobs {
            match job.proc.poll() {
                Some(exit_status) => {
                    println!("[{}] ({}) {:?} {}", job.jid, job.pid, exit_status, job.cmd);
                    v.insert(0, ind);
                }
                None => {}
            }

            ind += 1;
        }
        for it in v {
            self.jobs.remove(it);
        }
    }
}
