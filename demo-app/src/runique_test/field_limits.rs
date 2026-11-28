//! The bounds of a field (`min`, `max`, `min_length`…) as a project writes
//! them: the bound itself is accepted, one step past it is refused, with the
//! message given to the bound or, when none was given, the field's default.
use runique::chrono::NaiveTime;
use runique::forms::Limit;
use runique::prelude::*;

const CUSTOM: &str = "Refused by the project";

async fn error_for(mut field: impl FormField, value: &str) -> Option<String> {
    field.set_value(value);
    if field.validate().await {
        None
    } else {
        field.error().cloned()
    }
}

/// `inside` passes; `outside` is refused with the project's message when
/// `custom`, with a default one otherwise.
async fn check<F: FormField>(field: impl Fn() -> F, inside: &str, outside: &str, custom: bool) {
    assert_eq!(error_for(field(), inside).await, None, "{inside} is in");
    let error = error_for(field(), outside)
        .await
        .unwrap_or_else(|| panic!("{outside} is out"));
    if custom {
        assert_eq!(error, CUSTOM);
    } else {
        assert!(
            !error.is_empty() && error != CUSTOM,
            "default for {outside}"
        );
    }
}

fn date(day: &str) -> NaiveDate {
    day.parse().expect("a date")
}

fn time(hour: &str) -> NaiveTime {
    NaiveTime::parse_from_str(hour, "%H:%M").expect("a time")
}

fn datetime(at: &str) -> NaiveDateTime {
    NaiveDateTime::parse_from_str(at, "%Y-%m-%dT%H:%M").expect("a datetime")
}

#[tokio::test]
async fn date_bounds() {
    let field = || {
        DateField::new("day")
            .min(date("2026-01-10"), CUSTOM)
            .max(date("2026-01-20"), "")
    };
    check(field, "2026-01-10", "2026-01-09", true).await;
    check(field, "2026-01-20", "2026-01-21", false).await;
}

#[tokio::test]
async fn time_bounds() {
    let field = || {
        TimeField::new("hour")
            .min(time("08:00"), "")
            .max(time("18:00"), CUSTOM)
    };
    check(field, "08:00", "07:59", false).await;
    check(field, "18:00", "18:01", true).await;
}

#[tokio::test]
async fn datetime_bounds() {
    let field = || {
        DateTimeField::new("at")
            .min(datetime("2026-01-10T08:00"), CUSTOM)
            .max(datetime("2026-01-10T18:00"), "")
    };
    check(field, "2026-01-10T08:00", "2026-01-10T07:59", true).await;
    check(field, "2026-01-10T18:00", "2026-01-10T18:01", false).await;
}

#[tokio::test]
async fn duration_bounds() {
    let field = || {
        DurationField::new("length")
            .min_seconds(60, "")
            .max_seconds(3600, CUSTOM)
    };
    check(field, "60", "59", false).await;
    check(field, "3600", "3601", true).await;
}

#[tokio::test]
async fn text_length_bounds() {
    let field = || {
        TextField::text("name")
            .min_length(3, CUSTOM)
            .max_length(5, "")
    };
    check(field, "abc", "ab", true).await;
    check(field, "abcde", "abcdef", false).await;
}

#[tokio::test]
async fn number_bounds() {
    let field = || NumericField::integer("qty").min(1.0, CUSTOM).max(9.0, "");
    check(field, "1", "0", true).await;
    check(field, "9", "10", false).await;
}

/// The message travels with its bound, not in what the field's template reads.
#[test]
fn a_message_is_kept_with_its_bound() {
    let day = DateField::new("day")
        .min(date("2026-01-10"), CUSTOM)
        .max(date("2026-01-20"), "");
    assert_eq!(day.min_date, Some(Limit::new(date("2026-01-10"), CUSTOM)));
    assert_eq!(
        day.max_date.as_ref().and_then(|l| l.message.as_deref()),
        None
    );
    assert!(day.base.extra_context.is_empty());

    let qty = NumericField::integer("qty").min(1.0, CUSTOM).max(9.0, "");
    assert_eq!(qty.min_message.as_deref(), Some(CUSTOM));
    assert_eq!(qty.max_message, None);
    assert!(qty.base.extra_context.is_empty());
}
