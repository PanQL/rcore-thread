use super::*;

pub struct StaticScheduler {
    inner : Mutex<StaticSchedulerInner>,
}

struct StaticSchedulerInner {
    time_slice : usize,
    task_list : Vec<usize>,
    pre_task_list : Vec<usize>,
    infos : Vec<SSProcInfo>,
}

#[derive(Debug, Default, Copy, Clone)]
struct SSProcInfo {
    present: bool,  // 是否存在
    rest_slice: usize,  // 剩余运行周期
}

impl StaticSchedulerInner {
    pub fn new(time : usize) -> StaticSchedulerInner{
        StaticSchedulerInner{
            time_slice : time,
            task_list : Vec::new(),
            pre_task_list : Vec::new(),
            infos : Vec::new(),
        }
    }

    fn push(&mut self, tid : usize, time_slice : usize) {
        let tid = tid + 1;
        expand(&mut self.infos, tid);
        {
            let info = &mut self.infos[tid];
            info.present = true;
            info.rest_slice = time_slice;
        }
        self.pre_task_list.insert(0, tid);
    }

    fn pop(&mut self) -> Option<Tid> {
        let need_generate = self.task_list.is_empty();
        if need_generate {
            // regenerate task_list
            if let Some(items) = self.pre_task_list.get_mut(0..1) {
                self.task_list.append(items);
            }
        }
        self.task_list.get_mut(0)
    }

    fn tick(&mut self, current : Tid) -> bool {
        let current = current + 1;
        expand(&mut self.infos, current);
        assert!(!self.infos[current].present);

        let rest = &mut self.infos[current].rest_slice;
        if *rest > 0{
            *rest -= 1;
        } else {
            self.push(current - 1, 10); // FIXME how much should the default-time be
        }
        *rest == 0
    }
}
