const EMPTY_QUEUE = "Queue is empty!";
let toast_num = 0;
$(function () {
    $.ajaxSetup({
        contentType: 'application/json',
        dataType: 'html'
    });
    $(".rename").click(function(e) {
        console.log("rename clicked");
        let dat = JSON.stringify({
            "value": "ooh.m4a",
            "new_display_name": "ooh"
        });

        $.post("./rename", dat, () => {}, 'html')
        .done(function(response) {
            location.reload();
        })
        .fail(function(fail) {
            console.log("Failed: " + fail);
        });
    });
    $(".grid-item").click(function (e) {
        e.preventDefault();
        let val = {
            "value": $(this).attr('file_name'),
        };
        console.log(JSON.stringify(val));
        $.post("./sendReq", JSON.stringify(val), function (response) {
            alert("Mnogo dobro");
            console.log("Preslo je");
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
            });
    });

    $(".queue").click(function (e) {
        e.preventDefault();
        $.get("./queue", "", function (response) {
            // alert("Get Queue");
            console.log("Preslo je");
            console.log(response);
            generate_toast(response, response.success);
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
                generate_toast("err", "queue_fail");
            });
    });

    $(".skip").click(function (e) {
        e.preventDefault();
        $.get("./skip", "", function (response) {
            // alert("Get Queue");
            console.log("Preslo je");
            console.log(response);
            generate_toast(response, response.success);
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
                generate_toast("err", "skip_fail");
            });
    });

    $(".stop").click(function (e) {
        e.preventDefault();
        $.get("./stop", "", function (response) {
            // alert("Get Queue");
            console.log("Preslo je");
            console.log(response);
            generate_toast(response, response.success);
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
                generate_toast("err", "stop_fail");
            });
    });

    // volim biti retarderani
    function generate_toast(response, action) {
        let toast_class_selector = '.toast-' + toast_num;
        let toast_class = 'toast toast-' + toast_num;
        let toast = `<div class='` + toast_class + `'`;
        toast += `role='alert' data-delay='1000' aria-live='assertive' aria-atomic='true'>
            <div class='toast-header'>
                <strong class='mr-auto'>Queue</strong>
                <button type='button' class='ml-2 mb-1 close' data-dismiss='toast' aria-label='Close'>
                    <span aria-hidden='true'>&times;</span>
                </button>
            </div>`;
        switch (action) {
            case "queue_success": {
                if (response.queue.includes(EMPTY_QUEUE)) { 
                    toast += "<div class='toast-body'>" +  EMPTY_QUEUE + "</div>";
                } else {
                    for (let i = 0; i < response.queue.length; i++) {
                        if (i === 0) {
                            toast += "<div class='toast-body'>Now playing: " + response.queue[i] + "</div>";
                        } else {
                            toast += "<div class='toast-body'>" + (i) + ". " + response.queue[i] + "</div>";
                        }
                    }
                }
                break;
            }
            case "queue_fail": {
                toast += "<div class='toast-body'>Error getting queue</div>";
                break;
            }
            case "skip_success": {
                toast += "<div class='toast-body'>Skipping</div>";
                break;
            }
            case "skip_empty": {
                toast += "<div class='toast-body'>Queue is empty</div>";
                break;
            }
            case "skip_fail": {
                toast += "<div class='toast-body'>Failed skipping!</div>";
                break;
            }
            case "stop_success": {
                toast += "<div class='toast-body'>Stopping!</div>";
                break;
            }
            case "stop_empty": {
                toast += "<div class='toast-body'>There is nothing to stop!</div>";
                break;
            }
            case "stop_fail": {
                toast += "<div class='toast-body'>Failed to stop!</div>";
                break;
            }
        }
        toast += `</div>`
        $('.deni_tosta').append(toast);
        // registreraj listenera kaj ubi toasta z DOM-a
        $(toast_class_selector).on('hidden.bs.toast', function () {
            $(this).remove();
        });
        $(toast_class_selector).toast('show');
        toast_num++;
    }
});