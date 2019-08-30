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
        info!("push tid {}", tid);
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
        if need_generate {  // 重新生成执行表格
            let mut time_counter = 0;
            while time_counter <= self.time_slice && !self.pre_task_list.is_empty() {
                let item_index = self.pre_task_list.last().unwrap();
                let mut item = self.infos.get_mut(*item_index).unwrap();
                time_counter += item.rest_slice;
                item.present = false;
                self.task_list.push(self.pre_task_list.pop().unwrap());
            }
            warn!("new task_list : {:#?}", self.task_list);
        }
        let ret = if !self.task_list.is_empty() {
            let first = self.task_list.remove(0);
            self.infos[first].present = false;
            Some(first)
        }else{
            None
        };
        ret
    }

    fn tick(&mut self, current : Tid) -> bool {
        expand(&mut self.infos, current);
        assert!(!self.infos[current].present);

        let rest = &mut self.infos[current].rest_slice;
        let ret = if *rest > 0{
            *rest -= 1;
            info!("{} rest_slice {}", current, *rest);
            false
        } else {
            warn!("here");
            self.push(current - 1, 10); // FIXME how much should the default-time be
            true
        };
        info!("tick ret {}", ret);
        ret
    }
}

impl StaticScheduler {
    pub fn new(time_slice : usize) -> Self{
        let inner = StaticSchedulerInner::new(time_slice);
        StaticScheduler {
            inner : Mutex::new(inner),
        }
    }
}

impl Scheduler for StaticScheduler {
    fn push(&self, tid: usize) {
        self.inner.lock().push(tid, 10);
    }
    fn pop(&self, _cpu_id: usize) -> Option<usize> {
        self.inner.lock().pop()
    }
    fn tick(&self, current_tid: usize) -> bool {
        self.inner.lock().tick(current_tid)
    }
    fn set_priority(&self, _tid: usize, _priority: u8) {}
}
