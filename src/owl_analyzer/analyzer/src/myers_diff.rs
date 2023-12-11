#[derive(Clone, Debug)]
pub enum Different {
    // right idx
    Ins(usize),
    // left idx
    Del(usize),
    Eq((usize, usize)),
}

pub type DiffVec = Vec<Different>;

#[derive(Debug, Clone, Default)]
struct SolutionBuilder {
    diff: DiffVec,
    x: usize,
}

impl SolutionBuilder {
    pub fn add(&self, idx: usize) -> Self {
        let mut new_snake = self.clone();
        new_snake.diff.push(Different::Ins(idx));
        new_snake
    }

    pub fn del(&self, idx: usize) -> Self {
        let mut new_snake = self.clone();
        new_snake.diff.push(Different::Del(idx));
        new_snake.x += 1;
        new_snake
    }

    pub fn eq(&self, idx: (usize, usize)) -> Self {
        let mut new_snake = self.clone();
        new_snake.diff.push(Different::Eq(idx));
        new_snake.x += 1;
        new_snake
    }

    pub fn build(&mut self) -> DiffVec {
        std::mem::take(&mut self.diff)
    }
}

struct Snake {
    start: (isize, isize),
    end: (isize, isize),
    // vec: DiffVec,
    down: bool,
}

impl Snake {
    pub fn new(start: (isize, isize), end: (isize, isize), down: bool) -> Self {
        Self { start, end, down }
    }

    pub fn to_diffvec(&self) -> DiffVec {
        let mut builder = SolutionBuilder::default();
        let mut start = self.start;
        if self.down {
            builder = builder.add(start.1 as usize);
            start.1 += 1;
        } else {
            builder = builder.del(start.0 as usize);
            start.0 += 1;
        }
        while start != self.end {
            builder = builder.eq((start.0 as usize, start.1 as usize));
            start.0 += 1;
            start.1 += 1;
        }
        builder.build()
    }
}

pub fn myers_diff<T>(old: &[T], new: &[T]) -> Option<DiffVec>
where
    T: PartialEq,
{
    // both are empty
    if old.is_empty() && new.is_empty() {
        return Some(DiffVec::new());
    }

    if old.is_empty() {
        // let diff = DiffVec::with_capacity(old.len());
        let diff: Vec<_> = new
            .iter()
            .enumerate()
            .map(|(idx, _)| Different::Ins(idx))
            .collect();

        return Some(diff);
    } else if new.is_empty() {
        let diff: Vec<_> = old
            .iter()
            .enumerate()
            .map(|(idx, _)| Different::Del(idx))
            .collect();

        return Some(diff);
    }

    let n = old.len() as isize;
    let m = new.len() as isize;
    let max_step = n + m;
    let mut v = vec![0; 2 * max_step as usize];
    let mut snakes = Vec::new();

    for d in 0..=(max_step) {
        for k in (-d..=d).step_by(2) {
            let down = k == -d
                || (k != d && v[(k - 1 + max_step) as usize] < v[(k + 1 + max_step) as usize]);
            let k_prev = if down { k + 1 } else { k - 1 };

            let x_start = v[(k_prev + max_step) as usize];
            let y_start = x_start - k_prev;

            let mut x_final = if down { x_start } else { x_start + 1 };
            let mut y_final = x_final - k;

            // let mut snake = 0;
            while x_final < n && y_final < m && old[x_final as usize] == new[y_final as usize] {
                x_final += 1;
                y_final += 1;
                // snake += 1;
            }
            v[(k + max_step) as usize] = x_final;

            snakes.push(Snake::new((x_start, y_start), (x_final, y_final), down));

            if x_final >= n && y_final >= m {
                // return Some(v[k].build());
                let mut iter = snakes.iter().rev();
                let mut current = iter.next().unwrap();
                // res.extend(current.to_diffvec());
                let mut res = current.to_diffvec();
                if current.start == (0, -1) {
                    res.remove(0);
                }
                for tmp in iter {
                    if tmp.end == current.start {
                        current = tmp;
                        let mut new_res = current.to_diffvec();
                        new_res.extend(res);
                        res = new_res;
                        if current.start == (0, 0) {
                            break;
                        } else if current.start == (0, -1) {
                            res.remove(0);
                        }
                    }
                }
                return Some(res);
            }
        }
    }
    None
}

#[cfg(test)]
mod test {
    use super::myers_diff;

    #[test]
    pub fn test_myers() {
        // let l = [1, 2, 3, 4, 6, 7];
        let l = [];
        let r = [2, 4, 5, 6, 7, 8];
        // let r = [];
        let diff = myers_diff(&l, &r).unwrap();

        println!("{:?}", diff);
    }
}
