const EMPTY_QUEUE = "Queue is empty!";
let queue_toast_num = 0;

$(function () {
    $.ajaxSetup({
        contentType: 'application/json',
        dataType: 'html'
    });

    // display queue
    $(".queue").click(function (e) {
        e.preventDefault();
        $.get("./queue", "", function (response) {
            // alert("Get Queue");
            console.log("Preslo je");
            console.log(response);
            generate_queue_toast(response, response.success);
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
                generate_queue_toast("err", "queue_fail");
            });
    });

    // skip currently playing
    $(".skip").click(function (e) {
        e.preventDefault();
        $.get("./skip", "", function (response) {
            // alert("Get Queue");
            console.log("Preslo je");
            console.log(response);
            generate_queue_toast(response, response.success);
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
                generate_queue_toast("err", "skip_fail");
            });
    });

    // stop playing
    $(".stop").click(function (e) {
        e.preventDefault();
        $.get("./stop", "", function (response) {
            console.log("Preslo je");
            console.log(response);
            generate_queue_toast(response, response.success);
        }, 'json')
            .fail(function (x) {
                console.log("Neje preslo");
                generate_queue_toast("err", "stop_fail");
            });
    });
 // volim biti retarderani
    function generate_queue_toast(response, action) {
        let toast_class_selector = '.toast-' + queue_toast_num;
        let toast_class = 'toast toast-' + queue_toast_num;
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
        queue_toast_num++;
    }
});