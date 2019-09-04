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
    fn tick(&self, current_tid : Tid) -> bool {
        self.inner.lock().tick(current_tid)
    }
}

impl NaiveTTSchedulerInner{
    fn push(&mut self, tid : Tid, cycle : usize, offset : usize, max_time : usize) -> bool {
        let new_info = NaiveTTInfo::new(tid, cycle, offset, max_time);
        let mut ret = true;
        for info in self.infos.iter() {
            if info.check_conflict(&new_info) {
                ret = false;
                break;
            }
        }
        if ret {
            let new_time = self.time + (new_info.cycle - self.time % new_info.cycle) + new_info.offset;
            // 更新保存着TT线程开始时间的堆
            let index = self.infos.len();
            self.time_table.push((Reverse(new_time), index));
            self.infos.push(new_info);
        }
        ret
    }

    fn pop(&mut self) -> Option<Tid> {
        let mut ret = None;
        if let Some((Reverse(start_time), index)) = self.time_table.pop() {
            assert_eq!(start_time, self.time);
            let cycle = self.infos[index].cycle;
            let tid = self.infos[index].tid;
            let new_time_slice = self.infos[index].max_time;
            self.current = tid;
            self.time_slice = new_time_slice;
            self.time_table.push((Reverse(start_time + cycle), index));
            ret = Some(tid);
        }
        ret
    }

    fn tick(&mut self, _tid : Tid) -> bool {
        let mut ret = false;
        self.tick_counter += 1;
        if self.tick_counter == self.ticks_per_msec {
            self.tick_counter = 0;
            self.time += 1;
            if self.time_slice == 0 {
                if let Some((Reverse(start_time), _)) = self.time_table.peek() {
                    ret = *start_time == self.time;
                }
            }else{
                self.time_slice -= 1;
            }
        }
        ret
    }
}
