use super::*;

struct NaiveTTInfo{
    tid : Tid,
    cycle: usize,
    offset: usize,
    max_time: usize,
}

impl NaiveTTInfo{
    // return true if new TT_thread is conflict with this
    fn check_conflict(&self, other : &NaiveTTInfo) -> bool {
        let mut ret = false;
        let (lo, hi, min_cycle, begin, end, max_cycle) = if self.cycle < other.cycle {
                (self.offset, self.offset + self.max_time, self.cycle, 
                 other.offset, other.offset + other.max_time, other.cycle)
            }else{
                (other.offset, other.offset + other.max_time, other.cycle,
                self.offset, self.offset + self.max_time, self.cycle)
            };
        for i in (0..max_cycle).step_by(min_cycle) {
            let low = lo + i;
            let high = hi + i;
            if (begin <= low && end >= low) || (begin <= high && end >= high) {
                ret = true;
                break;
            }
        }
        ret
    }

    pub fn new(tid : usize, cycle : usize, offset : usize, max_time : usize) -> Self {
        NaiveTTInfo {
            tid, cycle, offset, max_time,
        }
    }
}

struct NaiveTTSchedulerInner{
    infos: Vec<NaiveTTInfo>,
    time_table: BinaryHeap<(Reverse<usize>, usize)>,
    time: usize,
    ticks_per_msec: usize,
    tick_counter: usize,
    current: usize,
    time_slice: usize,
}

pub struct NaiveTTScheduler {
    inner: Mutex<NaiveTTSchedulerInner>,
}

impl NaiveTTScheduler {
    pub fn new(ticks_per_msec: usize) -> Self {
        let inner = NaiveTTSchedulerInner{
            infos: Vec::default(),
            time_table: BinaryHeap::new(),
            time: 0,
            ticks_per_msec,
            tick_counter: 0,
            current: 0,
            time_slice: 0,
        };
        NaiveTTScheduler{
            inner: Mutex::new(inner),
        }
    }
}

impl TTScheduler for NaiveTTScheduler {
    fn push(&self, tid : Tid, cycle : usize, offset : usize, max_time : usize) -> bool{
        self.inner.lock().push(tid, cycle, offset, max_time)
    }
    fn pop(&self) -> Option<Tid> {
        self.inner.lock().pop()
    }
    fn tick(&self) -> bool {
        self.inner.lock().tick()
    }
    fn working(&self) -> bool {
        self.inner.lock().working()
    }
    fn stop(&self) {
        self.inner.lock().stop()
    }
}

impl NaiveTTSchedulerInner{
    fn push(&mut self, tid : Tid, cycle : usize, offset : usize, max_time : usize) -> bool {
        let new_info = NaiveTTInfo::new(tid, cycle, offset, max_time);
        let mut ret = true;
        error!("cycle {:#x}, offset {:#x}, max_time {:#x}", cycle, offset, max_time);
        if (max_time + offset) > cycle {
            ret = false;
            return ret;
        }
        for info in self.infos.iter() {
            if info.check_conflict(&new_info) {
                ret = false;
                break;
            }
        }
        if ret {
            error!("cycle {:#x}, offset {:#x}", new_info.cycle, new_info.offset);
            let new_time = self.time + (new_info.cycle - ( self.time % new_info.cycle )) + new_info.offset;
            error!("current time {:#x} new time {:#x}", self.time, new_time);
            // 更新保存着TT线程开始时间的堆
            let index = self.infos.len();
            self.time_table.push((Reverse(new_time), index));
            self.infos.push(new_info);
        }
        ret
    }

    fn pop(&mut self) -> Option<Tid> {
        let mut judge = false;  // 判断当前是否有新的TT线程需要执行
        if let Some((Reverse(start_time), _)) = self.time_table.peek() {
            judge = *start_time == self.time;
        }
        if judge {  // 有新的TT线程需要执行
            let (Reverse(start_time), index) = self.time_table.pop().unwrap();
            assert_eq!(start_time, self.time);
            let cycle = self.infos[index].cycle;
            error!("cycle is {:#x}", cycle);
            let tid = self.infos[index].tid;
            let new_time_slice = self.infos[index].max_time;
            self.current = tid;
            self.time_slice = new_time_slice;
            self.time_table.push((Reverse(start_time + cycle), index));
            Some(tid)
        }else{  // 没有新的TT线程需要执行
            None
        }
    }

    fn tick(&mut self) -> bool {
        let mut ret = false;
        self.tick_counter += 1;
        if self.tick_counter == self.ticks_per_msec {
            self.tick_counter = 0;
            self.time += 1;
            error!("current time {:#x} time_slice {:#x} is_empty : {}", self.time, self.time_slice, self.time_table.is_empty());
            if self.time_slice == 0 {   // 当前没有TT线程正在运行中
                // 查看time变化之后是否有新的需要执行的TT线程
                if let Some((Reverse(start_time), _)) = self.time_table.peek() {
                    error!("next tt_time {:#x}", *start_time);
                    ret = *start_time == self.time;
                }
            }else{  // 某一TT线程正在运行，将其运行ms数减1
                self.time_slice -= 1;
            }
        }
        if ret {
            error!("tt tick ret is true");
        }
        ret
    }

    fn working(&self) -> bool {
        self.time_slice > 0
    }

    fn stop(&mut self) {
        self.time_slice = 0
    }
}
