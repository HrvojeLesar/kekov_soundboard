const net = require('net');
const Discord = require('discord.js');
const { Buffer } = require('buffer');
const fs = require('fs');
const { exit } = require('process');
const { setTimeout } = require('timers');
const { time } = require('console');


const client = new Discord.Client();
// const guild = new Discord.Guild(client, "679094912179765271");

const HOST = 'localhost';
const PORT = 1337;
// const GUILD_ID = "173766075484340234"; // Vatikan server
//const GUILD_ID = "679094912179765271"; // T2 server

const MAIN_DIR= "../sounds/";
const MAIN_DIR_SPLIT = "/sounds/";

var queue = new Array();
let dispatcher;
let timeout;

// SOCKET RESPONSES
// 0 => Started playing (queue was empty before starting)
// 1 => Added to queue
// 2 => 
// 3 => Error joining channel. At least one person needs to be joined in a voice channel!
// 4 => Skipping
// 5 => Queue is empty
// 6 => Stop
// 7 => Nothing to stop

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

// client.login('NTY0OTIyMDAzMDE5MDA1OTUy.XwIEsA.wu8rTbP7tz243WqxS3TYIhdiL0o');
client.login(config.token);

client.on('ready', () => {
    console.log(`Logged in as ${client.user.tag}!`);
    // console.log(get_voice_channel(GUILD_ID));
});


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
    console.log("Stopping");
    if (queue.length > 1) {
        queue.length = 0;
        dispatcher.end();
        sock.write("6");
    } else {
        sock.write("7");
    }
}

function sound_manager(sock, data) {
    queue.push(data);
    clearTimeout(timeout);
    if (queue.length === 1) {
        let voice_channel = get_voice_channel(config.guild_id);
        if (voice_channel === undefined) {
            sock.write("3");
            queue.shift();
        } else {
            voice_channel.join().then(connection => play(connection));
            sock.write("0");
        }
    } else {
        // Added to queue
        sock.write("1");
    }
}

function send_queue_test(sock) {
    queue.push("test");
    queue.push("../sounds/");
    queue.push("Hit");
    queue.push("Ler");
    let len = 0;
    for (let i = 0; i < queue.length; i++) {
        len += Buffer.from(queue[i]).length;
    }
    // console.log(queue.join('?'));
    sock.write((len + (queue.length - 1)).toString());
    sock.write(queue.join("?"));

    queue.shift();
    queue.shift();
    queue.shift();
    queue.shift();
}

function send_queue(sock) {
    if (queue.length === 0) {
        sock.write("0");
        return 0;
    }
    let len = 0;
    let q = new Array();
    for (let i = 0; i < queue.length; i++) {
        len += Buffer.from(queue[i].value).length;
        q.push(queue[i].value);
    }
    sock.write(len + (queue.length - 1).toString());
    sock.write(q.join("?"));
}

function disconnect_bot(connection) {
    timeout = setTimeout(() => {
        dispatcher.destroy();
        connection.disconnect();
    }, 3000);
}
