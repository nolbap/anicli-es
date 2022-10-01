extern crate getopts;
use dialoguer::Input;
use getopts::Options;
use std::env;
use std::ops::Index;

mod mainfunctions;
use mainfunctions::{search_query, query_results, choose_anime, get_episodes, choose_index, mpv};

fn main() {

    // unix args
    let mut checked: bool = false;
    let args: Vec<String> = env::args().collect();
    let project_name = option_env!("PROJECT_NAME").unwrap_or("anicli-es");
    let mut options = Options::new();
    options.optflag("h", "help", "obtener ayuda para los comandos");

    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            panic!("{}", f.to_string())
        }
    };

    if matches.opt_present("h") {
        let brief = format!("Uso: {} [busqueda] [opciones]", project_name);
        print!("{}", options.usage(&brief));
        return;
    };

    // main loop
    loop { 

        let query: String =
            if !matches.free.is_empty() && !checked {
                checked = true;
                matches.free[0].clone()
            } else {
                Input::new().with_prompt("Buscar animé").interact().unwrap()
            };

        let resultados: Vec<Vec<String>> = query_results(search_query(query.trim().to_string()));

        if resultados[0].is_empty() {
            println!("No se encontró ningun animé con ese nombre.");
            continue;
        }

        let seleccion: i32 = choose_anime(&resultados);
        println!("Elegiste {}", resultados[0][seleccion as usize].as_str());

        let episodios: Vec<String> = get_episodes(resultados[2].index(seleccion as usize).to_string());

        if episodios.is_empty() {
            println!("No se encontrarón episodios, intenta con otro animé.");
            continue;
        } 

        mpv(
            resultados.index(0).index(seleccion as usize),
            &episodios,
            choose_index(episodios.len(), format!("episodio [1-{}]", episodios.len()).as_str()));
    }
}
