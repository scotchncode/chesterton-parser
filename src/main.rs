use std::error::Error as StdError;
use std::io::BufRead;
use std::str::FromStr;
use std::{fs::File, io::BufReader};

type BoxedError = Box<dyn StdError>;

fn main() -> Result<(), BoxedError> {
    let f = File::open("data.txt")?;
    let buf = BufReader::new(f);

    let mut entities: Vec<Entity> = vec![];
    let mut started = false;
    let mut buffer: Vec<String> = vec![];

    let skipables = ["", "THE MOVEABLE FEASTS", "APRIL"];

    for line in buf.lines() {
        let line = line?;

        let trimmed = line.trim();

        if trimmed == "NEW YEAR'S DAY" {
            started = true;
        }

        if started == false || skipables.contains(&trimmed) {
            continue;
        }

        buffer.push(line.clone());

        if buffer.len() < 3 {
            continue;
        }

        // APRIL 12th has underscores at the start of the line
        if trimmed.starts_with("_have_") {
            continue;
        }

        if trimmed.starts_with("'_") || trimmed.starts_with("_") {
            let (date, content, source) = match buffer[0].trim() {
                // JANUARY 6th has the name of the day on a lower line
                "JANUARY 6th" => (
                    buffer[0..2].join(" "),
                    &buffer[2..buffer.len() - 1],
                    &buffer[buffer.len() - 1],
                ),
                _ => (
                    buffer[0].to_string(),
                    &buffer[1..buffer.len() - 1],
                    &buffer[buffer.len() - 1],
                ),
            };

            entities.push(Entity {
                day: date.trim().to_string().parse().unwrap(),
                content: content.join("\n").trim_matches('\n').to_string(),
                source: source.trim().to_string(),
            });

            buffer = vec![];
        }
    }

    // for entity in entities.iter().take(6) {
    // for entity in entities.iter() {
    //     // thread::sleep(Duration::from_millis(250));
    //     println!("d: {:?}", entity.date);
    //     println!("s: {}", entity.source);
    //     println!("");
    // }

    // println!("{:?}", entities[0]);
    // println!("{:?}", entities[1]);
    // println!("{:?}", entities[2]);
    // println!("{:?}", entities[3]);
    // println!("{:?}", entities[4]);
    // println!("{:?}", entities[5]);
    println!("total entries: {}", entities.len());

    let output = File::create("output.csv")?;
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b'\t')
        .quote_style(csv::QuoteStyle::NonNumeric)
        .from_writer(output);

    let mut rows_written = 0;

    for entity in entities.into_iter() {
        let year = 2023;

        let Some(ref date) = entity.day.date else {
            continue;
        };

        let month: Month = match date.month.parse() {
            Ok(month) => month,
            Err(err) => {
                println!("failed to parse month to i32: {}", err);
                continue;
            }
        };

        let Some(date) = chrono::NaiveDate::from_ymd_opt(year, month as u32, date.day as u32) else {
            println!("unable to parse date: {:?}", entity.day);
            continue;
        };

        let date = date.to_string();
        let title = format!("{}", entity.day.to_string());
        let content = format!("{}\n\n{}\n\n{}", title, entity.content, entity.source);

        writer.write_record(&[date, title, content])?;
        rows_written += 1;
    }

    println!("entries written: {}", rows_written);

    writer.flush()?;

    Ok(())
}

#[derive(Debug)]
struct Entity {
    day: Day,
    content: String,
    source: String,
}

#[derive(Debug)]
struct Day {
    date: Option<Date>,
    name: Option<String>,
}

impl ToString for Day {
    fn to_string(&self) -> String {
        match (&self.date, &self.name) {
            (Some(date), Some(name)) => format!("{} - {}", date.to_string(), name),
            (None, Some(name)) => format!("{}", name),
            (Some(date), None) => format!("{}", date.to_string()),
            (None, None) => format!(""),
        }
    }
}

#[derive(Debug)]
struct Date {
    month: &'static str,
    day: i32,
}

impl ToString for Date {
    fn to_string(&self) -> String {
        let suffix = match self.day % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        };

        format!("{} {}{}", self.month, self.day, suffix)
    }
}

enum Month {
    JANUARY = 1,
    FEBRUARY = 2,
    MARCH = 3,
    APRIL = 4,
    MAY = 5,
    JUNE = 6,
    JULY = 7,
    AUGUST = 8,
    SEPTEMBER = 9,
    OCTOBER = 10,
    NOVEMBER = 11,
    DECEMBER = 12,
}

impl FromStr for Month {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let res = match s {
            "JANUARY" => Self::JANUARY,
            "FEBRUARY" => Self::FEBRUARY,
            "MARCH" => Self::MARCH,
            "APRIL" => Self::APRIL,
            "MAY" => Self::MAY,
            "JUNE" => Self::JUNE,
            "JULY" => Self::JULY,
            "AUGUST" => Self::AUGUST,
            "SEPTEMBER" => Self::SEPTEMBER,
            "OCTOBER" => Self::OCTOBER,
            "NOVEMBER" => Self::NOVEMBER,
            "DECEMBER" => Self::DECEMBER,
            _ => return Err(format!("failed to parse month: {}", s)),
        };

        Ok(res)
    }
}

impl FromStr for Day {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        let months = [
            "NEW YEAR'S DAY",
            "JANUARY",
            "FEBRUARY",
            "MARCH",
            "APRIL",
            "MAY",
            "JUNE",
            "JULY",
            "AUGUST",
            "SEPTEMBER",
            "OCTOBER",
            "NOVEMBER",
            "DECEMBER",
        ];

        let num_suffix = ["st", "nd", "rd", "th"];

        if let Some(month) = months
            .iter()
            .find(|month| s.trim().to_uppercase().starts_with(*month))
        {
            if month == &months[0] {
                return Ok(Day {
                    date: Some(Date {
                        month: months[1],
                        day: 1,
                    }),
                    name: Some(months[0].to_string()),
                });
            }

            let parts: Vec<&str> = s.split(" ").collect();

            if parts.len() == 1 {
                return Ok(Day {
                    date: Some(Date { month, day: 1 }),
                    name: None,
                });
            }

            if let Some(suffix) = num_suffix
                .iter()
                .find(|suff| parts[1].trim().ends_with(*suff))
            {
                let day: i32 = parts[1]
                    .replace(suffix, "")
                    .parse()
                    .map_err(|err| format!("failed to parse day: {}", err))?;

                if parts.len() == 2 {
                    return Ok(Day {
                        date: Some(Date { month, day }),
                        name: None,
                    });
                }

                return Ok(Day {
                    date: Some(Date { month, day }),
                    name: Some(parts[2..].join(" ")),
                });
            }
        }

        Ok(Day {
            date: None,
            name: Some(s.to_string()),
        })
    }
}
