use dialoguer::Input;
use regex::Regex;
use std::ops::Index;
use std::process::Command;

mod auxfunctions;

// recibe un largo(usize) en el cual elegir un índice y un String que dice a que corresponde el
// índice a elegir y entrega el input del usuario como entero i32
pub fn choose_index(lenght: usize, que: &str) -> i32 {

    if lenght == 1 {
        return 0
    }
    loop {
        let mut prompt: String = "Elige un ".to_string();
        prompt.push_str(que);

        let index: String = Input::new().with_prompt(prompt).interact().unwrap();

        let index: i32 = match index.trim().parse() {
            Ok(num) => num,
            Err(_) => {
                println!("Porfavor escribe un número.");
                continue;
            }
        };

        if index > lenght.try_into().unwrap() {
            println!("El índice seleccionado es invalido.");
            continue;
        } else if index < 1 {
            println!("El índice seleccionado es invalido.");
            continue;
        } else {
            return index - 1;
        }
    }
}


// recibe un String como busqueda y devuelve el código fuente del resultado
pub fn search_query(query: String) -> String {
    let mut url = "https://monoschinos2.com/buscar?q=".to_string();
    url.push_str(&query);

    auxfunctions::get_source(url).unwrap()
}

// recibe el código fuente de la busqueda como input y devuelve un vector de vectores por cada
// animé encontrado, con, el nombre del animé, categoría + año y su link
pub fn query_results(source: String) -> Vec<Vec<String>> {

    let get_titles = Regex::new("(<h3 class=\"seristitles\">)(.*)(</h3)").unwrap();
    let get_categories = Regex::new("(<span class=\"seriesinfo\">)(.*)(</span)").unwrap();
    let get_links = Regex::new("(<div class=\"col-md-4 col-lg-2 col-6\">\n<a href=\")(.*)(\">)").unwrap();

    let mut titles: Vec<String> = Vec::new();
    let mut categories: Vec<String> = Vec::new();
    let mut links: Vec<String> = Vec::new();


    for t in get_titles.captures_iter(&source) {
        titles.push(
            t.get(2)
            .unwrap()
            .as_str()
            .to_string()
            .replace("&#039;", "'")
            .replace("&amp;", "&")
        );
    }

    for c in get_categories.captures_iter(&source) {
        categories.push(
            c.get(2)
            .unwrap()
            .as_str()
            .to_string()
        );
    }

    for l in get_links.captures_iter(&source) {
        links.push(
            l.get(2)
            .unwrap()
            .as_str()
            .to_string()
        );
    }

    let tcl: Vec<Vec<String>> = vec![titles, categories, links];
    tcl

}

// printea los animés encontrados a la pantalla con indices a la izquierda
pub fn choose_anime(animelist: &Vec<Vec<String>>) -> i32 {
    if (animelist[0].len()) == 1 {
        return 0;
    } else {
        for n in 0..animelist[0].len() {
            println!("[{}] {} - {}", n+1, animelist[0][n], animelist[1][n]);
        }
    }

    let animes: String = format!("animé [1-{}]", animelist[0].len());
    choose_index(animelist[0].len(), animes.as_str())
}

// obtiene los links de los episodios de la página y los pone en un vector, si falta un link
// guarda un link vacío y anota que episodio falta en otro vector
pub fn get_episodes(url: String) -> Vec<String> {
    let source = auxfunctions::get_source(url).unwrap();
    let fetch_links_and_indexes = Regex::new("(<div class=\"col-item\".*data-episode=\")(.*)(\">.*\n.*<a href=\")(.*)(\">)").unwrap();
    
    let mut links: Vec<String> = Vec::new();
    let mut missing: Vec<i32> = Vec::new();
    let mut index: i32 = 1; 

    for n in fetch_links_and_indexes.captures_iter(&source) {
        let episode_no: i32 = n.get(2).unwrap().as_str().parse().unwrap();
        if index == episode_no {
            links.push(n.get(4).unwrap().as_str().to_string());
        } else {
            for x in 0..(episode_no-index) {
                links.push("".to_string());
                missing.push(index+x);
            }
            index = episode_no;
        }
        index += 1;
    }

    return links;
}

// obtiene los links del mp4 + el reproductor de video embedded si es que el servidor lo requiere,
// guarda los links en un vector de vectores en que, el primero corresponde al servidor zeus, el
// segundo uqload y el tercero a videobin
pub fn episode_link_scrapper(url: String) -> Vec<Vec<String>> {

    println!(
        "{esc}[2J{esc}[1;1HObteniendo información del video.\n",
        esc = 27 as char
    );

    let is_puj = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3JlcHJvLm1vbm9zY2hpbm9zMi5jb20vYXF1YS9hbT91cmw9)(.*)(\">puj</a></li>)").unwrap();
    let mut pdone: bool = false;

    let is_zeus = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3d3dy5zb2xpZGZpbGVzLmNvbS9l)(.*)(\">.eus</a></li>)").unwrap();
    let mut zdone: bool = false;
  
    let is_fembed = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3d3dy5mZW1iZWQuY29tL3Yv)(.*)(\">fembed2</a></li>)").unwrap();
    let mut fdone: bool = false;

    let is_videobin = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3ZpZGVvYmluLmNv)(.*)(\">videobin</a></li>)").unwrap();
    let mut vdone: bool = false;

    let is_uqload = Regex::new("(aHR0cHM6Ly9tb25vc2NoaW5vczIuY29tL3JlcHJvZHVjdG9yP3VybD1odHRwczovL3VxbG9hZC5jb20v)(.*)(\">uqload</a></li>)").unwrap();
    let mut qdone: bool = false;

    let mut plink: Vec<String> = vec!["0".to_string()];
    let mut zlink: Vec<String> = vec!["0".to_string()];
    let mut flink: Vec<String> = vec!["0".to_string()];
    let mut vlink: Vec<String> = vec!["1".to_string()];
    let mut qlink: Vec<String> = vec!["1".to_string()];

    let video_source: String = auxfunctions::get_source(url).unwrap();

    loop {
        if is_puj.is_match(&video_source) && !pdone {

            plink.push(auxfunctions::decode_base64(
                is_puj
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
            ));

            pdone = true;
        } else if is_zeus.is_match(&video_source) && !zdone {
        
            let mut embedded_link: String = "https://www.solidfiles.com/e".to_string();
            embedded_link.push_str(&auxfunctions::decode_base64(
                is_zeus
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
            ));

            // regex para obtener el archivo mp4 (zeus)
            let get_video_link = Regex::new("(\"streamUrl\":\")(.*)(\",\"nodeName)").unwrap();
            let embedded_source: String = auxfunctions::get_source(embedded_link).unwrap();

            if get_video_link.is_match(&embedded_source) {
                zlink.push(get_video_link
                    .captures(&embedded_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
                    );
                }

            zdone = true;
        } else if is_fembed.is_match(&video_source) && !fdone {

            let mut postreq_link: String = "https://fembed-hd.com/api/source/".to_string();
            postreq_link.push_str(&auxfunctions::decode_base64(
                is_fembed
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
            ));

            let get_video_link = Regex::new("(,.\"file\":\"https:././fvs.io./)(.*)(\",\"label\":\"720p\")").unwrap();
            let postreq_source: String = auxfunctions::post_request(postreq_link).unwrap();
            let mut link: String = "https://fvs.io/".to_string();

            if get_video_link.is_match(&postreq_source) {
                link.push_str(get_video_link
                        .captures(&postreq_source)
                        .unwrap()
                        .get(2)
                        .unwrap()
                        .as_str()
                );

                flink.push(link);
                
            }

            fdone = true;
        } else if is_videobin.is_match(&video_source) && !vdone {

            let mut embedded_link: String = "https://videobin.co".to_string();
            embedded_link.push_str(&auxfunctions::decode_base64(
                is_videobin
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string(),
            ));
 
            vlink.push(embedded_link);

            // regex para obtener el archivo mp4 (videobin)
            let get_video_link = Regex::new("(sources: .\".*\")(.*)(\".*)").unwrap();
            let embedded_source: String = auxfunctions::get_source(vlink.index(1).to_string()).unwrap();

            if get_video_link.is_match(&embedded_source) {
                vlink.push(get_video_link
                    .captures(&embedded_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
                    );
                }

            vdone = true;
        } else if is_uqload.is_match(&video_source) && !qdone {
            
            let mut embedded_link: String = "https://uqload.com/".to_string();
            embedded_link.push_str(&auxfunctions::decode_base64(
                is_uqload
                    .captures(&video_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
            ));

            qlink.push(embedded_link);

            // regex para obtener el archivo mp4 (uqload)
            let get_video_link = Regex::new("(sources: .\")(.*)(\".)").unwrap();
            let embedded_source: String = auxfunctions::get_source(qlink.index(1).to_string()).unwrap();

            if get_video_link.is_match(&embedded_source) {
                qlink.push(get_video_link
                    .captures(&embedded_source)
                    .unwrap()
                    .get(2)
                    .unwrap()
                    .as_str()
                    .to_string()
                    );
                }

            qdone = true;
        } else {
            break;
        }
    }
    return vec![plink, zlink, flink, vlink, qlink]
}

fn getargs(links: Vec<Vec<String>>) -> String {
    for n in 0..links.len() {
        if links[n][0] == "0" && links[n].len() == 2 {
            return format!("setsid -f mpv --really-quiet \"{}\"",
                links[n].index(1).to_string())
        } else if links[n][0] == "1" && links[n].len() == 3 {
            return format!("setsid -f mpv --really-quiet --http-header-fields=\"Referer:{}\" \"{}\"",
                links[n].index(1).to_string(),
                links[n].index(2).to_string())
        } else {
            String::new()
        };
    };

    String::new()
}

pub fn mpv(nombre: &String, links: &[String], episodio: i32) {
    
    let episode_links: Vec<Vec<String>> = episode_link_scrapper(links.index(episodio as usize).to_string());
    let args: String = getargs(episode_links);

    if !args.is_empty() {
        let mpv_command = Command::new("sh")
            .arg("-c")
            .arg(&args)
            .spawn();

        drop(mpv_command);

        if links.len() == 1 {
            println!("Viendo \"{}\".\n", nombre);
        } else {
            println!(
                "Viendo \"{}\", episodio {}.\n",
                nombre,
                episodio + 1
            );
        }
        controller(episodio, links.to_vec(), nombre);
    } else {
        println!("No se encontró ningun servidor útil.");
        mpv(nombre, &links, choose_index(links.len(), format!("episodio [1-{}]", links.len()).as_str()));
    };
}

fn controller(index: i32, links: Vec<String>, nombre: &String) {
    let mut case: i8 = 0;
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

        prompt.push_str("[r] Ver de nuevo\n[o] Seleccionar otro episodio\n[b] Buscar otro anime\n[q] Salir\nEscoge una opción");

        let opcion: String = Input::new().with_prompt(prompt).interact().unwrap();
        let opcion = opcion.trim().to_string();

        if opcion.to_lowercase() == "q" {
            std::process::exit(0);
        } else if opcion.to_lowercase() == "b" {
            break();
        } else if opcion.to_lowercase() == "r" {
            mpv(nombre, &links, index);
        } else if opcion.to_lowercase() == "o" {
            mpv(nombre, &links, choose_index(links.len(), format!("episodio [1-{}]", links.len()).as_str()));
        } else if case == 1 && opcion.to_lowercase() == "s" {
            mpv(nombre, &links, index+1);
        } else if case == 2 && opcion.to_lowercase() == "a" {
            mpv(nombre, &links, index - 1);
        } else if case == 3 && opcion.to_lowercase() == "s" {
            mpv(nombre, &links, index + 1);
        } else if case == 3 && opcion.to_lowercase() == "a" {
            mpv(nombre, &links, index - 1);
        } else {
            println!("Escoge una opción valida.\n");
            continue;
        }
    }
}
