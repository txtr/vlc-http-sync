use std::cmp::Ordering;
use std::env;
use std::{thread, time};

const AUTH: &str = "Basic OjEyMzQ="; // base64 encoded password for string ":1234"
const ACCEPTABLE_SLIP: u64 = 2; // in seconds
const SLAVE_PLAYER_URL: &str = "http://localhost:8080/requests/status.xml";

#[derive(Debug, Clone)]
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
    let response = client.get(url).header("Authorization", AUTH).send();

    match response {
        Ok(r) => {
            let text = r.text().unwrap();
            return Ok(Player {
                state: text
                    .split("</state>")
                    .next()
                    .unwrap()
                    .split("<state>")
                    .last()
                    .unwrap()
                    .parse::<String>()
                    .unwrap(),
                looping: text
                    .split("</loop>")
                    .next()
                    .unwrap()
                    .split("<loop>")
                    .last()
                    .unwrap()
                    .parse::<bool>()
                    .unwrap(),
                random: text
                    .split("</random>")
                    .next()
                    .unwrap()
                    .split("<random>")
                    .last()
                    .unwrap()
                    .parse::<bool>()
                    .unwrap(),
                repeat: text
                    .split("</repeat>")
                    .next()
                    .unwrap()
                    .split("<repeat>")
                    .last()
                    .unwrap()
                    .parse::<bool>()
                    .unwrap(),
                length: text
                    .split("</length>")
                    .next()
                    .unwrap()
                    .split("<length>")
                    .last()
                    .unwrap()
                    .parse::<u64>()
                    .unwrap(),
                position: text
                    .split("</position>")
                    .next()
                    .unwrap()
                    .split("<position>")
                    .last()
                    .unwrap()
                    .parse::<f64>()
                    .unwrap(),
                time_elapsed: text
                    .split("</time>")
                    .next()
                    .unwrap()
                    .split("<time>")
                    .last()
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

fn correct(client: &reqwest::blocking::Client, master_player: &Player, slave_player: &Player) {
    let slip: u64 = match master_player.time_elapsed.cmp(&slave_player.time_elapsed) {
        Ordering::Less => slave_player.time_elapsed - master_player.time_elapsed,
        Ordering::Greater => master_player.time_elapsed - slave_player.time_elapsed,
        Ordering::Equal => 0,
    };
    println!("slip = {} sec", slip);
    if slip > ACCEPTABLE_SLIP {
        seek(client, master_player.time_elapsed);
        println!("corrected")
    }
}

fn sync(client: &reqwest::blocking::Client, master_player: Player, slave_player: Player) {
    if master_player.state == "stopped" && slave_player.state != "stopped" {
        stop(client);
    } else if slave_player.state == "stopped" {
        println!("Instruction: Play the same file as other vlc player.")
    } else if master_player.length == slave_player.length {
        correct(client, &master_player, &slave_player);
        if master_player.state == "playing" && slave_player.state != "playing" {
            play(client);
        } else if master_player.state == "paused" && slave_player.state != "paused" {
            pause(client);
        }
    } else {
        println!("Error: Files are of different length. Stalling sync. untill resolved.");
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let master_player_url: String = format!("{}{}", &args[1], "/requests/status.xml");
    let master_player_url: &str = &master_player_url[..];

    let client = reqwest::blocking::Client::new();
    prep(&client, SLAVE_PLAYER_URL);
    prep(&client, master_player_url);
    loop {
        println!("syncing...");
        sync(
            &client,
            get_player(&client, master_player_url).unwrap(),
            get_player(&client, SLAVE_PLAYER_URL).unwrap(),
        );
        thread::sleep(time::Duration::from_secs(1));
    }
}
