use druid::{widget::SizedBox, Data, Widget};
use druid_enums::Matcher;

#[derive(Clone, Data)]
struct A;

#[derive(Clone, Data)]
struct B;

#[allow(dead_code)]
#[derive(Clone, Data, Matcher)]
enum AB {
    A(A),
    B(B),
}

#[test]
fn return_type() {
    fn inner() -> impl Widget<AB> {
        AB::matcher()
    }
    inner();
}

#[test]
fn with_all_variants() {
    AB::matcher()
        .a(SizedBox::<A>::empty())
        .b(SizedBox::<B>::empty());
}

#[test]
fn with_with_default() {
    AB::matcher()
        .a(SizedBox::<A>::empty())
        .default(SizedBox::<AB>::empty());

    #[rustfmt::skip]
    AB::matcher()
        .a(SizedBox::<A>::empty())
        .default_empty();
}

#[test]
fn generated_matcher_name() {
    ABMatcher::new()
        .a(SizedBox::<A>::empty())
        .b(SizedBox::<B>::empty());
}
