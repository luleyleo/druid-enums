# Druid Enums

This allows deriving a `Matcher` for any Rust enum.
The `Matcher` allows matching a `druid::Widget` to each variant of the enum.

This is what it should look like, once it is done:

```rust
#[derive(Clone, Data, Matcher)]
enum Event {
    Click(u32, u32),
    Key(char),
}

fn event_widget1() -> impl Widget<Event> {
    Event::matcher()
        .click(Label::dynamic(|data, _| {
            format!("x: {}, y: {}", data.0, data.1)
        ))
        .key(Label::dynamic(|data, _| {
            format!("key: {}", data))
        })
}

fn event_widget2() -> impl Widget<Event> {
    Event::matcher()
        .key(Label::dynamic(|data, _| {
            format!("key: {}", data))
        })
        .default(Label::new("Unhandled Event"))
    }
}

fn event_widget3() -> impl Widget<Event> {
    // Will emit warning for missing variant `Event::Click` at runtime
    Event::matcher()
        .key(Label::dynamic(|data, _| {
            format!("key: {}", data))
        })
    }
}
```

