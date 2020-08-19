let play_toast_num = 0;

$(function () {
    $.ajaxSetup({
        contentType: 'application/json',
        dataType: 'html'
    });

    // play clicked item
    $(".grid-container").on("click", ".playable-item", function(e) {
        e.preventDefault();
        let val = {
            "value": $(this).attr('file_name'),
        };
        console.log(JSON.stringify(val));
        $.post("./sendReq", JSON.stringify(val), function (response) {
            console.log("Preslo je");
            generate_play_toast(response);
        }, 'json')
            .fail(function (err) {
                console.log(err);
                generate_play_toast("err");
        });
    });

    function generate_play_toast(response) {
        let toast_class_selector = '.toast-' + play_toast_num;
        let toast_class = 'toast toast-' + play_toast_num;
        let toast = `<div class='` + toast_class + `'`;
        toast += `role='alert' data-delay='1000' aria-live='assertive' aria-atomic='true'>`;
        if (response === "err") {
            toast += `
                <div class='toast-header' style='background: #f94825; color: white;'>
                    <strong class='mr-auto'>Error</strong>
                    <button type='button' class='ml-2 mb-1 close' data-dismiss='toast' aria-label='Close'>
                    <span aria-hidden='true'>&times;</span>
                    </button>
                </div>`;
            toast += `<div class='toast-body'>Error connecting to bot!</div>`;
        } else if (response.error === undefined) {
            toast += `
                <div class='toast-header' style='background: #75fc53; color: white;'>
                    <strong class='mr-auto'>S U C C</strong>
                    <button type='button' class='ml-2 mb-1 close' data-dismiss='toast' aria-label='Close'>
                    <span aria-hidden='true'>&times;</span>
                    </button>
                </div>`;
                    
            switch(response.success) {
                case "playing": {
                    toast += "<div class='toast-body'>Playing</div>";
                    break;
                }
                case "added": {
                    toast += "<div class='toast-body'>Added to queue</div>";
                    break;
                }
            }
        } else {
            toast += `
                <div class='toast-header' style='background: #f94825; color: white;'>
                    <strong class='mr-auto'>Error</strong>
                    <button type='button' class='ml-2 mb-1 close' data-dismiss='toast' aria-label='Close'>
                    <span aria-hidden='true'>&times;</span>
                    </button>
                </div>`;

            switch(response.error) {
                case "error joining": {
                    toast += "<div class='toast-body'>All voice channels are empty</div>";
                    break;
                }
                case "unknown error": {
                    toast += "<div class='toast-body'>Unknown error! Prekini mi trti bota!</div>";
                    break;
                }
            }
        }
        toast += `</div>`;
        $('.deni_tosta').append(toast);
        // registreraj listenera kaj ubi toasta z DOM-a
        $(toast_class_selector).on('hidden.bs.toast', function () {
            $(this).remove();
        });
        $(toast_class_selector).toast('show');
        play_toast_num++;
    }
});