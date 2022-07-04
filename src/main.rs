use anyhow::{anyhow, bail, Result as AnyhowResult};
use itertools::Itertools;
use scraper::{Html, Selector};
use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod addresses;
mod script_mapper;

use addresses::Addresses;
use script_mapper::Mapper;

// https://elektrodistribucija.rs/NoviSad_Dan_0_Iskljucenja.htm

static BEOGRAD_DAY_0: &str = "https://elektrodistribucija.rs/Dan_0_Iskljucenja.htm";

static BEOGRAD_DAY_1: &str = "https://elektrodistribucija.rs/Dan_1_Iskljucenja.htm";

static BEOGRAD_DAY_2: &str = "https://elektrodistribucija.rs/Dan_2_Iskljucenja.htm";

static BEOGRAD_DAY_3: &str = "https://elektrodistribucija.rs/Dan_3_Iskljucenja.htm";

static BEOGRAD: &[&str] = &[BEOGRAD_DAY_0, BEOGRAD_DAY_1, BEOGRAD_DAY_2, BEOGRAD_DAY_3];

#[tokio::main]
async fn main() -> AnyhowResult<()> {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    let text_mapper = Mapper::new();

    let table_selector = Selector::parse("table:nth-child(2)").map_err(|e| anyhow!("{e:?}"))?;
    let tr_selector: Selector =
        Selector::parse("tr:not(:first-child)").map_err(|e| anyhow!("{e:?}"))?;
    let td_selector = Selector::parse("td").map_err(|e| anyhow!("{e:?}"))?;

    for url in BEOGRAD.iter() {
        let body = reqwest::get(*url).await?.text().await?;

        let document = Html::parse_document(&body);

        if let Some(data_table) = document.select(&table_selector).next() {
            for (i, row) in data_table.select(&tr_selector).enumerate() {
                let mut data_sel = row.select(&td_selector);
                let columns = data_sel.next().and_then(|d| {
                    data_sel.next().and_then(|t| {
                        data_sel.next().map(|s| {
                            (
                                d.text().map(str::trim).join(""),
                                t.text().map(str::trim).join(""),
                                s.text().map(str::trim).join(""),
                            )
                        })
                    })
                });

                if let Some((d, t, s)) = columns {
                    let transformed: String = text_mapper.transoform(&s);
                    let x = Addresses::parse(transformed.as_str()).map_err(|e| anyhow!("{e}"))?;
                    println!("{}\t{t}\t{x:?}", text_mapper.transoform(&d));
                    println!("\n\n-----------\n");
                } else {
                    tracing::warn!("malformed row #{i}: {row:?}");
                }
            }
        } else {
            bail!("the page does not contain the data table");
        }
    }

    Ok(())
}
