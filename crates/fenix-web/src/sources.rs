//! Bootstrap source routing, not answer URLs or an external search service.
//! These roots only help cold corpora; actual article links must be observed.
use crate::{fetch, search};

pub(crate) fn roots(query: &str) -> Vec<String> {
    let terms = search::terms(query);
    let sources: &[(&[&str], &str)] = &[
        (&["react", "useeffect", "usestate"], "https://react.dev/"),
        (&["next", "nextjs"], "https://nextjs.org/docs"),
        (&["tokio"], "https://docs.rs/tokio/latest/tokio/"),
        (&["rust", "cargo"], "https://doc.rust-lang.org/book/"),
        (
            &["python", "asyncio", "pathlib"],
            "https://docs.python.org/3/",
        ),
        (
            &["javascript", "css", "html", "flexbox", "webassembly"],
            "https://developer.mozilla.org/en-US/docs/Web",
        ),
        (&["typescript"], "https://www.typescriptlang.org/docs/"),
        (&["sqlite"], "https://www.sqlite.org/docs.html"),
        (
            &["postgresql", "postgres"],
            "https://www.postgresql.org/docs/current/",
        ),
        (&["docker"], "https://docs.docker.com/"),
        (&["kubernetes", "kubectl"], "https://kubernetes.io/docs/"),
        (&["git"], "https://git-scm.com/docs"),
        (&["context7"], "https://context7.com/docs"),
        (
            &[
                "nasa", "mars", "marte", "jupiter", "saturn", "saturno", "planet", "planetas",
                "moon", "luna", "eclipse", "solar",
            ],
            "https://science.nasa.gov/",
        ),
        (
            &["climate", "clima", "climático", "ipcc"],
            "https://www.ipcc.ch/",
        ),
        (
            &[
                "who", "oms", "diabetes", "malaria", "dengue", "vaccines", "vacunas", "health",
                "salud",
            ],
            "https://www.who.int/health-topics/",
        ),
        (
            &[
                "inflation",
                "inflación",
                "economy",
                "economía",
                "poverty",
                "pobreza",
                "gdp",
                "pib",
            ],
            "https://www.worldbank.org/en/topic",
        ),
        (
            &["perú", "peru", "reniec", "sunat", "migraciones"],
            "https://www.gob.pe/",
        ),
        (&["arxiv", "quantum", "cuántica"], "https://arxiv.org/"),
    ];
    let mut ranked: Vec<_> = sources
        .iter()
        .map(|(names, root)| {
            (
                names
                    .iter()
                    .filter(|name| terms.iter().any(|term| term == **name))
                    .count(),
                *root,
            )
        })
        .filter(|(score, _)| *score > 0)
        .collect();
    ranked.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    let mut roots = Vec::new();
    // An explicit hostname in the query is also a source hint, never a guessed article.
    for word in query.split_whitespace() {
        let word = word.trim_matches(['(', ')', ',', ';']);
        if word.contains('.') && !word.contains('@') {
            let candidate = if word.starts_with("https://") || word.starts_with("http://") {
                word.to_owned()
            } else {
                format!("https://{word}/")
            };
            if let Ok(url) = fetch::public_url(&candidate)
                && url.host_str().is_some_and(|host| host.rsplit('.').next().is_some_and(|tld| tld.len() >= 2 && tld.chars().all(char::is_alphabetic)))
                // Product spelling is not an actual domain hint.
                && !matches!(word.to_lowercase().as_str(), "next.js" | "node.js" | "vue.js")
            {
                roots.push(url.to_string());
            }
        }
    }
    roots.extend(ranked.into_iter().map(|(_, root)| root.to_owned()));
    roots.dedup();
    roots.truncate(4);
    roots
}
