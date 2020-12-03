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
const WEBSOCKET_PORT = 2222;
//const GUILD_ID = "679094912179765271"; // T2 server

const MAIN_DIR= "../sounds/";
const MAIN_DIR_SPLIT = "/sounds/";

var queue = new Array();
let dispatcher;
let timeout;

// SOCKET RESPONSES
// 0 => Started playing (queue was empty before starting)
// 1 => Added to queue
// 2 => Send queue
// 3 => Error joining channel. At least one person needs to be joined in a voice channel!
// 4 => Skipping
// 5 => Queue is empty
// 6 => Stop
// 7 => Nothing to stop
// 8 => Matijos je banati
// 9 => Nembrem ga banati

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


client.on('ready', () => {
    console.log(`Logged in as ${client.user.tag}!`);
    update_voice_monitor();
    // console.log(get_voice_channel(GUILD_ID));
    // send_queue_test();
});

client.on("message", (message) => {
    if (!message.author.bot) {
        if (message.content.toLowerCase() === "!stop") {
            console.log("STOP command");
            stop(undefined);
        }
    }
});



client.on('voiceStateUpdate', () => {
    update_voice_monitor();
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
            if (channels[i].members.size == 1 && channels[i].members.array()[0].user.bot) {
                return undefined;
            }
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
        sock.write("7");
        return 0;
    }

    if (queue.length >= 1) {
        queue.length = 0;
        if (sock != undefined) {
            dispatcher.end();
            sock.write("6");
        } else {
            console.log("Stopping");
            dispatcher.end();
        }
    } else {
        dispatcher.end();
        sock.write("7");
    }
}

function sound_manager(sock, data) {
    let voice_channel = get_voice_channel(config.guild_id);

    if (voice_channel === undefined) {
        sock.write("3");
        return;
    }

    clearTimeout(timeout);
    // if (queue.length === 0) {
    //     queue.push(data);
    //     voice_channel.join().then(connection => play(connection));
    //     sock.write("0");
    // } else {
    //     queue.push(data);
    //     sock.write("1");
    // }

    if (!client.voice.connections.get(config.guild_id)) {
        queue.push(data);
        voice_channel.join().then(connection => play(connection));
        sock.write("0");
    } else if (queue.length === 0) {
        queue.push(data);
        voice_channel.join().then(connection => play(connection));
        sock.write("0");
    } else {
        queue.push(data);
        sock.write("1");
    }
}

function send_queue(sock) {

    if (queue.length == 0) {
        sock.write("5");
        return;
    }

    let q = {
        "queue": []
    };

    for (let i = 0; i < queue.length; i++) {
        q.queue.push(queue[i].value);
    }

    let json_string = JSON.stringify(q);
    
    sock.write("2" + json_string);
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

function update_voice_monitor() {
    let voice_channel = get_voice_channel(config.guild_id);

    if (voice_channel === undefined) {
        let data = {
            "members": null,
            "avatars": null,
            "channel": null
        };
        send_data_to_voice_monitor(data);
        return; 
    }
    let voice_channel_name = voice_channel.name;
    let members = voice_channel.members.array();

    let members_names = new Array();
    let members_avatars = new Array();
    for (let i = 0; i < members.length; i++) {
        let nickname = members[i].nickname;
        let avatar = members[i].user.avatarURL();
        if (nickname) {
            members_names.push(nickname);
        } else {
            members_names.push(members[i].user.username);
        }

        if (avatar) {
            members_avatars.push(avatar);
        } else {
            members_avatars.push(members[i].user.defaultAvatarURL);
        }
    }

    let data = {
        "members": members_names,
        "avatars": members_avatars,
        "channel": voice_channel_name
    };

    send_data_to_voice_monitor(data);
}

function send_data_to_voice_monitor(data) {
    let send_nicknames = net.createConnection({port: WEBSOCKET_PORT}, () => {
        console.log("Sent members data:\n", data);
        send_nicknames.write(JSON.stringify(data));
        send_nicknames.end();
    });

    send_nicknames.on('error', (e) => {
        console.error(e);
    }); 
}

client.login(config.token);
