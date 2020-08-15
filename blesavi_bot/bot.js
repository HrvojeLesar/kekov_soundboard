const net = require('net');
const Discord = require('discord.js');
const { Buffer } = require('buffer');
const fs = require('fs');
const { exit } = require('process');
const { setTimeout } = require('timers');
const { time } = require('console');


const client = new Discord.Client();

const HOST = 'localhost';
const PORT = 1337;
//const GUILD_ID = "679094912179765271"; // T2 server

const MAIN_DIR= "../sounds/";
const MAIN_DIR_SPLIT = "/sounds/";

var queue = new Array();
let dispatcher;
let timeout;

// SOCKET RESPONSES
// 0 => Started playing (queue was empty before starting)
// 1 => Added to queue
// 2 => RETARDERANI SAM
// 3 => Error joining channel. At least one person needs to be joined in a voice channel!
// 4 => Skipping
// 5 => Queue is empty
// 6 => Stop
// 7 => Nothing to stop
// 8 => Matijos je banati
// 9 => Nembrem ga banati

// flow
// sharding ?
// check if playing (connected)
// if playing add to queue
// else join first channel and play sound

// config start
let config;

if (!fs.existsSync("./config.json")) {
    console.log("Please create a config.json file");
    exit(0);
} else {
    try {
        config = JSON.parse(fs.readFileSync('./config.json', 'utf-8'));
    } catch (err) {
        console.error(err)
        exit(0);
    }
}

client.login(config.token);

client.on('ready', () => {
    console.log(`Logged in as ${client.user.tag}!`);
    // console.log(get_voice_channel(GUILD_ID));
    // send_queue_test();
});

// client.on('presenceUpdate', (_, newPres) => {
//     if (newPres.guild.id === config.guild_id && newPres.userID === "132286945031094272") {
//         console.log("Nekaj se je premenilo");
//     }
// });

net.createServer((sock) => {
    console.log("Connected: " + sock.remoteAddress + ":" + sock.remotePort);
    sock.on('data', (data) => {
        let parsed_data = JSON.parse(data.toString());
        console.log(parsed_data);

        switch (parsed_data.command) {
            case 'play': {
                sound_manager(sock, parsed_data);
                break;
            }
            case 'queue': {
                send_queue(sock);
                break;
            }
            case 'skip': {
                skip(sock);
                break;
            }
            case 'stop': {
                stop(sock);
                break;
            }
            case 'banaj': {
                banajMatijosa(sock);
                break;
            }
            default: {
                console.log("Invalid command");
                sock.write("3");
            }
        }
    });

    sock.on('close', (close) => {
        console.log("Closed: " + sock.remoteAddress + ' ' + sock.remotePort);
    });

    sock.on('timeout', (timeout) => {
        console.log(timeout);
    });

    sock.on('error', (error) => {
        console.log("Error: " + error);
    })

}).listen(PORT, HOST);

console.log("Server listening on " + HOST + ':' + PORT);

function get_voice_channel(guild_id) {
    let channels = client.guilds.cache.get(guild_id).channels.cache.array();
    for (let i = 0; i < channels.length; i++) {
        if (channels[i].type === 'voice' && channels[i].members.size > 0) {
            return channels[i];
        }
    }
    return undefined;
}

function play(connection) {
    console.log("Playing: " + MAIN_DIR + queue[0].file_name);
    dispatcher = connection.play((MAIN_DIR + queue[0].file_name), {volume: 1});

    dispatcher.on('finish', () => {
        queue.shift();
        if(queue[0]) {
            play(connection);
        } else {
            disconnect_bot(connection);
        }
    });
}

function skip(sock) {
    if (queue.length < 1) {
        sock.write("5");
    } else {
        console.log("Skipping: " + queue[0].file_name);
        dispatcher.end();
        sock.write("4");
    }
}

function stop(sock) {
    if (dispatcher == null) {
        console.log("Dispatcher not initialized");
        return 0;
    }

    console.log("Stopping");
    if (queue.length >= 1) {
        queue.length = 0;
        dispatcher.end();
        sock.write("6");
    } else {
		dispatcher.end();
        sock.write("7");
    }
}

function sound_manager(sock, data) {
    queue.push(data);
    clearTimeout(timeout);
    if (queue.length === 1) {
        let voice_channel = get_voice_channel(config.guild_id);
        if (voice_channel === undefined) {
            // 3 => Error joining channel. At least one person needs to be joined in a voice channel!
            sock.write("3");
            queue.shift();
        } else {
            voice_channel.join().then(connection => play(connection));
            // 0 => Started playing (queue was empty before starting)
            sock.write("0");
        }
    } else {
        // 1 => Added to queue
        sock.write("1");
    }
}

function send_queue_test() {
    queue.push("test");
    queue.push("../sounds/");
    queue.push("Hit");
    queue.push("Ler");

    let q = {
        "success": "queue_success",
        "queue": []
    };
    for (let i = 0; i < queue.length; i++) {
        q.queue.push(queue[i]);
    }

    let json_string = JSON.stringify(q);

    console.log(json_string);
    // console.log(q.to_string().getBytes("UTF-8"));

    queue.shift();
    queue.shift();
    queue.shift();
    queue.shift();
}

// function send_queue(sock) {
//     if (queue.length === 0) {
//         sock.write("0");
//         return 0;
//     }
//     let len = 0;
//     let q = new Array();
//     for (let i = 0; i < queue.length; i++) {
//         len += Buffer.from(queue[i].value).length;
//         q.push(queue[i].value);
//     }
//     sock.write(len + (queue.length - 1).toString());
//     sock.write(q.join("?"));
// }

function send_queue(sock) {
    let q = {
        "queue": []
    };
    for (let i = 0; i < queue.length; i++) {
        q.queue.push(queue[i].value);
    }

    let json_string = JSON.stringify(q);
    
    sock.write(Buffer.from(json_string).length.toString());
    sock.write(json_string);
}


function disconnect_bot(connection) {
    timeout = setTimeout(() => {
        dispatcher.destroy();
        connection.disconnect();
    }, 3000);
}

function banajMatijosa(sock) {
    client.guilds.cache.get(config.guild_id).members.ban("252114544485335051")
        .then(user => {
            console.log("Banned ${user.username || user.id || user}");
            sock.write("8");
        })
        .catch((error) => {
            console.log(error);
            sock.write("9");
        });
}
