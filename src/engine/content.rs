pub(super) fn title(room: &str) -> &'static str {
    match room {
        "generator" => "Generator hall",
        "workshop" => "Workshop",
        "fuel" => "Fuel store",
        "office" => "Compliance",
        "control" => "Control room",
        "break" => "Break room",
        "stores" => "Lost property",
        "records" => "Records",
        _ => unreachable!(),
    }
}

pub(super) fn stations(room: &str) -> &'static [(&'static str, &'static str, char)] {
    match room {
        "generator" => &[
            ("generator", "Start generator", 'G'),
            ("handle", "Pull auxiliary handle", 'H'),
            ("poster", "Inspect motivation", '?'),
        ],
        "workshop" => &[
            ("tool", "Take chain release", ')'),
            ("locker", "Inspect award-winning locker", ']'),
        ],
        "fuel" => &[
            ("can", "Release fuel can", '!'),
            ("pump", "Fill fuel can", 'P'),
        ],
        "office" => &[
            ("form", "Take inspection form", '?'),
            ("stamp", "Approve inspection", 'S'),
            ("office-note", "Inspect office policy", '?'),
        ],
        "control" => &[
            ("dial", "Adjust acceptable range", 'D'),
            ("breaker", "Toggle breaker", 'B'),
            ("fan", "Crank ventilation", 'F'),
        ],
        "break" => &[
            ("mug", "Take company mug", 'u'),
            ("kettle", "Dispense motivation", 'K'),
            ("sip", "Take mandatory coffee break", '_'),
        ],
        "stores" => &[("lost", "Inspect lost property", ']')],
        "records" => &[("records", "Inspect previous attempts", '?')],
        _ => unreachable!(),
    }
}
