$(function() {
    var conn = null;

    function connect() {
        disconnect();
        var wsUri = (window.location.protocol=='https:'&&'wss://'||'ws://')+window.location.host + '/ws-voice-monitor/';
        conn = new WebSocket(wsUri);
        console.log('Connecting');
        conn.onopen = function() {
            console.log("Connected");
        };
        conn.onmessage = function(e) {
            console.log("Received" + e.data);
            let sock_data = JSON.parse(e.data);
            $(".sock-voice-container").empty();
            if (sock_data.members === null) {
                $(".sock-voice-container").append(`
                    <div class="sock-voice-channel">All voice channels are empty</div>
                `);
            } else {
                $(".sock-voice-container").append(`
                    <div class="sock-voice-channel">` + sock_data.channel + `</div>
                `);
                for(let i = 0; i < sock_data.members.length; i++) {
                    $(".sock-voice-container").append(`
                        <div class="sock-voice-container-item">
                            <img src=` + sock_data.avatars[i] + ` alt=` + sock_data.members[i] +  ` class="sock-voice-user-avatar">
                            <div class="sock-voice-username">` + sock_data.members[i] + `</div>
                        </div>
                    `);
                }
            }
        };
        conn.onclose = function() {
            console.log("Disconnected");
            conn = null;
        }
    }

    function disconnect() {
        if (conn != null) {
            console.log("Disconnecting...");
            conn.close();
            conn = null;
        }
    }

    connect();
});