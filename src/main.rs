extern crate getopts;
use dialoguer::Input;
use getopts::Options;
use regex::Regex;
use std::convert::TryInto;
use std::env;
use std::ops::Index;
use std::process::Command;
use std::str;

fn main() {
    //Unix args (Terminar...)
    let mut checked: bool = false;
    let args: Vec<String> = env::args().collect();
    let project_name = option_env!("PROJECT_NAME").unwrap_or("anicli-es");
    let mut options = Options::new();
    options.optflag("h", "help", "print this help menu");

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };
    if matches.opt_present("h") {
        let brief = format!("Uso: {} [search query] [options]", project_name);
        print!("{}", options.usage(&brief));
        return;
    };

    //Main loop
    loop {
        let mut _query_source = String::new();

        //Buscar animé si se encuentra un argumento al principio
        if !matches.free.is_empty() && !checked {
            checked = true;
            let query: String = matches.free[0].clone();
            _query_source = search_query(query).unwrap();

        //Buscar animé
        } else {
            let query: String = Input::new().with_prompt("Buscar anime").interact().unwrap();
            let query = query.trim().to_string();
            _query_source = search_query(query).unwrap();
        };

        let titles_categories_and_links: Vec<Vec<String>> =
            get_titles_categories_and_links(_query_source);

        if titles_categories_and_links[0].is_empty() {
            println!("No se encontró ningun animé con ese nombre.\n");
            continue;
        }

        let choice: u32 = escoger_anime(
            titles_categories_and_links[0].len(),
            &titles_categories_and_links,
        );

        let anime_link: String = titles_categories_and_links[2]
            .index(choice as usize)
            .to_string();
        let anime_source: String = get_source(anime_link).unwrap();
        let links: Vec<String> = get_episodes_and_links(anime_source);

        println!(
            "Escogiste: \"{}\"",
            titles_categories_and_links[0][choice as usize]
        );

        mpv(
            titles_categories_and_links.index(0),
            &links,
            escoger_episodio(links.len()),
            choice,
        );
    }
}

fn get_source(url: String) -> Result<String, ureq::Error> {
    let source = ureq::get(&url).call()?.into_string()?;
    Ok(source)
}

fn decode(input: String) -> String {
    let bytes = base64::decode(input).unwrap();
    let decoded_string = match str::from_utf8(&bytes) {
        Ok(v) => v,
        Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
    };

    decoded_string.to_string()
}

fn search_query(query: String) -> Result<String, ureq::Error> {
    let mut url = "https://monoschinos2.com/buscar?q=".to_string();

    url.push_str(&query);
    get_source(url)
}

fn choose_index(lenght: usize, que: String) -> u32 {
    loop {
        let mut prompt: String = "Escoge un ".to_string();
        prompt.push_str(&que);

        let index: String = Input::new().with_prompt(prompt).interact().unwrap();

        let index: u32 = match index.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Porfavor escribe un número.");
                continue;
            }
        };

        if index > (lenght + 1).try_into().unwrap() {
            println!("El indice seleccionado es invalido.");
            continue;
        } else if index < 1 {
            println!("El indice seleccionado es invalido.");
            continue;
        } else {
            return index - 1;
        }
    }
}

fn escoger_episodio(linkslen: usize) -> u32 {
    if linkslen == 1 {
        0
    } else {
        let n_episodios: String = format!("episodio [1-{}]", linkslen);
        choose_index(linkslen, n_episodios)
    }
}

fn escoger_anime(animeslen: usize, animeslist: &Vec<Vec<String>>) -> u32 {
    if animeslen == 1 {
        0
    } else {
        for n in 0..(animeslen) {
            println!("[{}] {} - {}", n + 1, animeslist[0][n], animeslist[1][n]);
        }

        let animes: String = format!("anime [1-{}]", animeslen);
        choose_index(animeslen, animes)
    }
}
fn get_titles_categories_and_links(source: String) -> Vec<Vec<String>> {
    let get_titles = Regex::new("(<h5 class=\"seristitles\">)(.*)(</h5)").unwrap();
    let get_links =
        Regex::new("(<div class=\"col-md-4 col-lg-2 col-6\">\n<a href=\")(.*)(\">)").unwrap();
    let get_category = Regex::new("(<span class=\"seriesinfo\">)(.*)(</span)").unwrap();

    let mut titulos: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();
    let mut categories: Vec<String> = Vec::new();

    for t in get_titles.captures_iter(&source) {
        titulos.push(
            t.get(2)
                .unwrap()
                .as_str()
                .to_string()
                .replace("&#039;", "'")
                .replace("&amp;", "&"),
        );
    }

    for c in get_category.captures_iter(&source) {
        categories.push(c.get(2).unwrap().as_str().to_string());
    }

    for l in get_links.captures_iter(&source) {
        links.push(l.get(2).unwrap().as_str().to_string());
    }

    let titles_categories_and_links: Vec<Vec<String>> = vec![titulos, categories, links];
    titles_categories_and_links
}

fn get_video_links(link: String) -> Vec<String> {
    println!(
        "{esc}[2J{esc}[1;1HObteniendo información del video.\n",
        esc = 27 as char
    );
    let is_uqload = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3VxbG9hZC5jb20v)(.*)(=\">uqload</a></li>)").unwrap();
    let mut qerror: bool = false;
    let is_videobin = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3ZpZGVvYmluLmNv)(.*)(=\">videobin</a></li>)").unwrap();
    let mut verror: bool = false;

    let mut links = Vec::new();
    let video_source: String = get_source(link).unwrap();

    loop {
        if is_uqload.is_match(&video_source) && !qerror {
            let get_actual_video_link = Regex::new("(sources: .\")(.*)(\".)").unwrap();

            let mut embedded_video_link: String = "https://uqload.com/".to_string();
            embedded_video_link.push_str(&decode(
                is_uqload
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string(),
            ));

            links.push(embedded_video_link);

            let embedded_video_source: String = get_source(links.index(0).to_string()).unwrap();

            if get_actual_video_link.is_match(&embedded_video_source) {
                let actual_video_link: String = get_actual_video_link
                    .captures(&embedded_video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string();
                links.push(actual_video_link);
                break;
            } else {
                qerror = true;
                continue;
            }
        } else if is_videobin.is_match(&video_source) && !verror {
            let get_actual_video_link = Regex::new("(sources: .\".*\",\")(.*)(\".*)").unwrap();

            let mut embedded_video_link: String = "https://videobin.co".to_string();
            embedded_video_link.push_str(&decode(
                is_videobin
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string(),
            ));

            links.push(embedded_video_link);

            let embedded_video_source: String = get_source(links.index(0).to_string()).unwrap();

            if get_actual_video_link.is_match(&embedded_video_source) {
                let actual_video_link: String = get_actual_video_link
                    .captures(&embedded_video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string();
                links.push(actual_video_link);
                break;
            } else {
                verror = true;
                continue;
            }
        } else {
            println!("No se encontró ningun servidor útil");
            std::process::exit(0);
        }
    }
    links
}

fn get_episodes_and_links(source: String) -> Vec<String> {
    let get_links =
        Regex::new("(<div class=\"col-item\" data-episode=\".*\">\n<a href=\")(.*)(\">)").unwrap();
    let mut links: Vec<String> = Vec::new();
    let mut rlinks: Vec<String> = Vec::new();

    for l in get_links.captures_iter(&source) {
        links.push(l.get(2).unwrap().as_str().to_string());
    }

    for n in (0..(links.len())).rev() {
        rlinks.push(links.index(n).to_string())
    }
    rlinks
}

fn mpv(names: &[String], links: &[String], episodio: u32, a_index: u32) {
    let video_links = get_video_links(links.index(episodio as usize).to_string());
    let mpv_command = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "setsid -f mpv --really-quiet --http-header-fields=\"Referer:{}\" \"{}\"",
            video_links.index(0).to_string(),
            video_links.index(1).to_string()
        ))
        .spawn();

    drop(mpv_command);
    if links.len() == 1 {
        println!("Viendo \"{}\".\n", names[a_index as usize]);
    } else {
        println!(
            "Viendo \"{}\", episodio {}.\n",
            names[a_index as usize],
            episodio + 1
        );
    }
    controller(episodio, links.to_vec(), names.to_vec(), a_index);
}

fn controller(index: u32, links: Vec<String>, names: Vec<String>, a_index: u32) {
    let mut case: u8 = 0;
    let linkslen = links.len();

    loop {
        let mut prompt = String::from("");

        if index == 0 && index + 1 == linkslen.try_into().unwrap() {
        } else if index == 0 {
            prompt.push_str("[s] Siguiente episodio\n");
            case = 1;
        } else if index + 1 == linkslen.try_into().unwrap() {
            prompt.push_str("[a] Anterior episodio\n");
            case = 2;
        } else {
            prompt.push_str("[a] Anterior episodio\n[s] Siguiente Episodio\n");
            case = 3;
        }

        prompt.push_str("[b] Buscar otro anime\n[q] Salir\nEscoge una opción");

        let opcion: String = Input::new().with_prompt(prompt).interact().unwrap();
        let opcion = opcion.trim().to_string();

        if opcion.to_lowercase() == "q" {
            std::process::exit(0);
        } else if opcion.to_lowercase() == "b" {
            main();
        } else if case == 1 && opcion.to_lowercase() == "s" {
            mpv(&names, &links, index + 1, a_index);
        } else if case == 2 && opcion.to_lowercase() == "a" {
            mpv(&names, &links, index - 1, a_index);
        } else if case == 3 && opcion.to_lowercase() == "s" {
            mpv(&names, &links, index + 1, a_index);
        } else if case == 3 && opcion.to_lowercase() == "a" {
            mpv(&names, &links, index - 1, a_index);
        } else {
            println!("Escoge una opción valida.\n");
            continue;
        }
    }
}
