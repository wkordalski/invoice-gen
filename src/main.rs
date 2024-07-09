use chrono::NaiveDate;
use handlebars::{Helper, Context, RenderContext, Output, Handlebars, HelperResult};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use temp_dir::TempDir;

fn main() -> anyhow::Result<()> {
    let mut args = <CliArgs as clap::Parser>::parse();
    let input = std::io::read_to_string(&mut args.input)?;

    let invoice: Invoice = toml::from_str(&input)?;
    let sum = invoice.sum();

    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(handlebars::no_escape);
    handlebars.register_helper("pln", Box::new(pln_helper));
    handlebars.register_helper("mul", Box::new(mul_helper));
    handlebars.register_helper("inc", Box::new(inc_helper));
    handlebars.register_helper("escape_dot_space", Box::new(escape_dot_space_helper));
    let rendered = handlebars.render_template(include_str!("./template.hbs"), &TemplateData { invoice, sum })?;

    let temp = TempDir::new()?;
    let tex = temp.child("invoice.tex");
    let pdf = tex.with_extension("pdf");
    std::fs::write(&tex, rendered)?;
    if !std::process::Command::new("pdflatex").current_dir(temp.path()).arg(tex).status()?.success() {
        return Err(anyhow::anyhow!("pdflatex failed"));
    }
    println!("Opening {}", pdf.display());
    std::io::copy(&mut std::fs::File::open(&pdf)?, &mut args.output)?;

    Ok(())
}

#[derive(clap::Parser)]
struct CliArgs {
    input: clio::Input,
    output: clio::Output,
}

#[derive(Serialize)]
struct TemplateData {
    invoice: Invoice,
    sum: Decimal,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Invoice {
    invoice: InvoiceInfo,
    buyer: Subject,
    seller: Subject,
    payment: Payment,
    products: Vec<Product>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct InvoiceInfo {
    id: String,
    #[serde(with = "date_format")]
    created: NaiveDate,
    done: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Subject {
    name: String,
    street: String,
    region: String,
    nip: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Payment {
    #[serde(with = "date_format")]
    date: NaiveDate,
    account: String,
    bank: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Product {
    name: String,
    unit: String,
    quantity: Decimal,
    price: Decimal,
    pkd: String,
}

impl Invoice {
    fn sum(&self) -> Decimal {
        self.products.iter().map(|p| p.price * p.quantity).sum()
    }
}

fn pln_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let price = Decimal::from_str_exact(h.param(0).unwrap().value().as_str().unwrap()).unwrap();
    out.write(&format!("{price:.2}").replace('.', ","))?;
    Ok(())
}

fn mul_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let a = Decimal::from_str_exact(h.param(0).unwrap().value().as_str().unwrap()).unwrap();
    let b = Decimal::from_str_exact(h.param(1).unwrap().value().as_str().unwrap()).unwrap();
    out.write(&(a*b).to_string())?;
    Ok(())
}

fn inc_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let a = h.param(0).unwrap().value().as_u64().unwrap();
    out.write(&(a+1).to_string())?;
    Ok(())
}

fn escape_dot_space_helper(h: &Helper, _: &Handlebars, _: &Context, _: &mut RenderContext, out: &mut dyn Output) -> HelperResult {
    let a = h.param(0).unwrap().value().as_str().unwrap();
    out.write(&a.replace(". ", ".\\ "))?;
    Ok(())
}

mod date_format {
    use chrono::NaiveDate;
    use serde::{de::Error, Deserialize, Deserializer, Serializer};

    pub(crate) fn serialize<S>(date: &NaiveDate, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_str(&date.format("%Y/%m/%d").to_string())
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>
    {
        let value = <toml::Value as Deserialize<'de>>::deserialize(deserializer)?;
        let toml::Value::Datetime(datetime) = value else { return Err(D::Error::custom("not a date")) };
        let Some(date) = datetime.date else { return Err(D::Error::custom("not a date")) };
        NaiveDate::from_ymd_opt(date.year.into(), date.month.into(), date.day.into())
            .ok_or_else(|| D::Error::custom("invalid date"))
    }
}
