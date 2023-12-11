use crate::myers_diff::{myers_diff, Different};

pub trait Align: PartialEq
where
    Self: Sized,
{
    fn empty_copy(&self) -> Option<Self>;
}

pub trait VecAlign {
    type Inner;

    fn align<'a>(
        &'a self,
        other: &'a Self,
    ) -> (Vec<Option<&'a Self::Inner>>, Vec<Option<&'a Self::Inner>>);
}

pub trait VecAlignOwned {
    type Inner;

    fn align_own(self, other: Self) -> (Vec<Option<Self::Inner>>, Vec<Option<Self::Inner>>);
}

impl<T> VecAlign for Vec<T>
where
    T: PartialEq,
{
    type Inner = T;

    fn align<'a>(
        &'a self,
        other: &'a Self,
    ) -> (Vec<Option<&'a Self::Inner>>, Vec<Option<&'a Self::Inner>>) {
        align_addr_impl(self, other)
    }
}

impl<T> VecAlignOwned for Vec<T>
where
    T: PartialEq + Default,
{
    type Inner = T;

    fn align_own(self, other: Self) -> (Vec<Option<Self::Inner>>, Vec<Option<Self::Inner>>) {
        align_addr_owned_impl(self, other)
    }
}

fn align_addr_impl<'a, T: PartialEq>(
    left: &'a Vec<T>,
    right: &'a Vec<T>,
) -> (Vec<Option<&'a T>>, Vec<Option<&'a T>>) {
    let diffs = myers_diff(&left, &right).unwrap();

    let mut l_ref = Vec::new();
    let mut r_ref = Vec::new();

    for d in diffs {
        match d {
            Different::Eq((li, ri)) => {
                l_ref.push(Some(left.get(li).unwrap()));
                r_ref.push(Some(right.get(ri).unwrap()));
            }
            Different::Ins(ri) => {
                r_ref.push(Some(right.get(ri).unwrap()));
                l_ref.push(None);
            }
            Different::Del(li) => {
                l_ref.push(Some(left.get(li).unwrap()));
                r_ref.push(None);
            }
        }
    }

    (l_ref, r_ref)
}

fn align_addr_owned_impl<T: PartialEq + Default>(
    mut left: Vec<T>,
    mut right: Vec<T>,
) -> (Vec<Option<T>>, Vec<Option<T>>) {
    // use myers to compare
    let diffs = myers_diff(&left, &right).unwrap();

    let mut l_ref = Vec::new();
    let mut r_ref = Vec::new();

    for d in diffs {
        match d {
            // eq
            Different::Eq((li, ri)) => {
                l_ref.push(Some(std::mem::take(left.get_mut(li).unwrap())));
                r_ref.push(Some(std::mem::take(right.get_mut(ri).unwrap())));
            }
            Different::Ins(ri) => {
                l_ref.push(None);
                r_ref.push(Some(std::mem::take(right.get_mut(ri).unwrap())));
            }
            Different::Del(li) => {
                l_ref.push(Some(std::mem::take(left.get_mut(li).unwrap())));
                r_ref.push(None);
            }
        }
    }

    (l_ref, r_ref)
}

#[cfg(test)]
mod test {
    use crate::align::align_addr_impl;

    #[test]
    pub fn test_align() {
        let left = Vec::from([0, 1, 2, 4, 5, 3, 7, 9, 3]);
        let right = Vec::from([1, 3, 5, 3, 8, 9]);

        println!("{:?}", left);
        println!("{:?}", right);

        // align_addr_impl(&mut left, &mut right);
        let (l, r) = align_addr_impl(&left, &right);

        println!("{:?}", l);
        println!("{:?}", r);

        assert_eq!(l.len(), r.len());
    }
}
