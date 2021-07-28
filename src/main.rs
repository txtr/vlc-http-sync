use reqwest;
use std::cmp::Ordering;
use std::{thread, time};
use std::env;

const SECOND: time::Duration = time::Duration::from_secs(1);

const AUTH: &str = "Basic OjEyMzQ=";
const ACCEPTABLE_SLIP: u64 = 3; // in seconds
const SLAVE_PLAYER_URL: &str = "http://localhost:8080/requests/status.xml";
// const MASTER_PLAYER_URL: &str = "http://c7dd9f13aa0d.in.ngrok.io/requests/status.xml";


#[derive(Debug)]
struct Player {
    state: String,
    looping: bool,
    random: bool,
    repeat: bool,
    length: u64,
    position: f64,
    time_elapsed: u64,
}

fn get_player(client: &reqwest::blocking::Client, url: &str) -> Result<Player, reqwest::Error> {
    let response = client
        .get(url)
        .header("Authorization", AUTH)
        .send();

    match response {
        Ok(r) => {
            let text = r.text().unwrap();
            return Ok(Player {
                state: text
                    .clone()
                    .split("<state>")
                    .nth(1)
                    .unwrap()
                    .split("</state>")
                    .nth(0)
                    .unwrap()
                    .parse::<String>()
                    .unwrap(),
                looping: text
                    .clone()
                    .split("<loop>")
                    .nth(1)
                    .unwrap()
                    .split("</loop>")
                    .nth(0)
                    .unwrap()
                    .parse::<bool>()
                    .unwrap(),
                random: text
                    .clone()
                    .split("<random>")
                    .nth(1)
                    .unwrap()
                    .split("</random>")
                    .nth(0)
                    .unwrap()
                    .parse::<bool>()
                    .unwrap(),
                repeat: text
                    .clone()
                    .split("<repeat>")
                    .nth(1)
                    .unwrap()
                    .split("</repeat>")
                    .nth(0)
                    .unwrap()
                    .parse::<bool>()
                    .unwrap(),
                length: text
                    .clone()
                    .split("<length>")
                    .nth(1)
                    .unwrap()
                    .split("</length>")
                    .nth(0)
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
                position: text
                    .clone()
                    .split("<position>")
                    .nth(1)
                    .unwrap()
                    .split("</position>")
                    .nth(0)
                    .unwrap()
                    .parse::<f64>()
                    .unwrap(),
                time_elapsed: text
                    .clone()
                    .split("<time>")
                    .nth(1)
                    .unwrap()
                    .split("</time>")
                    .nth(0)
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
            });
        }
        Err(e) => return Err(e),
    };
}

fn stop(client: &reqwest::blocking::Client) {
    client
        .get(format!("{}?command=pl_stop", SLAVE_PLAYER_URL))
        .header("Authorization", AUTH)
        .send()
        .expect("Err: seek failed");
}

fn seek(client: &reqwest::blocking::Client, time: u64) {
    client
        .get(format!("{}?command=seek&val={}", SLAVE_PLAYER_URL, time))
        .header("Authorization", AUTH)
        .send()
        .expect("Err: seek failed");
}

fn play(client: &reqwest::blocking::Client) {
    client
        .get(format!("{}?command=pl_play", SLAVE_PLAYER_URL))
        .header("Authorization", AUTH)
        .send()
        .expect("Err: play failed");
}

fn pause(client: &reqwest::blocking::Client) {
    play(client); // Workaround for toggle nature of pause.
    client
        .get(format!("{}?command=pl_pause", SLAVE_PLAYER_URL))
        .header("Authorization", AUTH)
        .send()
        .expect("Err: pause failed");
}

fn random_off(client: &reqwest::blocking::Client, url: &str) {
    if get_player(client, url).unwrap().random {
        client
            .get(format!("{}?command=pl_random", SLAVE_PLAYER_URL))
            .header("Authorization", AUTH)
            .send()
            .expect("Err: random_off failed");
    }
}

fn repeat_off(client: &reqwest::blocking::Client, url: &str) {
    if get_player(client, url).unwrap().repeat {
        client
            .get(format!("{}?command=pl_repeat", SLAVE_PLAYER_URL))
            .header("Authorization", AUTH)
            .send()
            .expect("Err: loop_on failed");
    }
}

fn loop_on(client: &reqwest::blocking::Client, url: &str) {
    if !(get_player(client, url).unwrap().looping) {
        client
            .get(format!("{}?command=pl_loop", SLAVE_PLAYER_URL))
            .header("Authorization", AUTH)
            .send()
            .expect("Err: loop_on failed");
    }
}

fn prep(client: &reqwest::blocking::Client, url: &str) {
    random_off(client, url);
    loop_on(client, url);
    repeat_off(client, url);
}

fn sync(client: &reqwest::blocking::Client, master_player: Player, slave_player: Player) {
    if master_player.length == slave_player.length {
        if master_player.state == "stopped" {
            if slave_player.state != "stopped" {
                stop(client);
            }
        } else /* if master_player.state == "playing" || master_player.state == "paused" */ {
            let slip: u64 = match master_player.time_elapsed.cmp(&slave_player.time_elapsed) {
                Ordering::Less => slave_player.time_elapsed - master_player.time_elapsed,
                Ordering::Greater => master_player.time_elapsed - slave_player.time_elapsed,
                Ordering::Equal => 0,
            };
            println!("{}", slip);
            if slip > ACCEPTABLE_SLIP {
                seek(client, master_player.time_elapsed);
            }

            if master_player.state == "playing" && slave_player.state != "playing" {
                play(client);
            } else if master_player.state == "paused" && slave_player.state != "paused" {
                pause(client);
            }
        }
    } else {
        println!("Err: Files are of different length.")
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let master_player_url: String = format!("{}{}", &args[1], "/requests/status.xml");
    let master_player_url: &str = &master_player_url[..];

    
    // let slave_player_url = &args[2];

    let client = reqwest::blocking::Client::new();
    prep(&client, SLAVE_PLAYER_URL);
    prep(&client, master_player_url);
    loop {
        println!("syncing...");
        sync(&client, get_player(&client, master_player_url).unwrap(), get_player(&client, SLAVE_PLAYER_URL).unwrap());
        thread::sleep(SECOND);
    }
}
