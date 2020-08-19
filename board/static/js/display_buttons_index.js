$(function () {
    $.ajaxSetup({
        contentType: 'application/json',
        dataType: 'json'
    });

    $.get("./get-buttons", "", function(response) {
        let result = [];
        for (var i in response.paths) {
            result.push(response.paths[i]);
        }
        result.sort((a, b) => { 
            if(a.display_name < b.display_name) { return -1; }
            if(a.display_name > b.display_name) { return 1; }
            return 0;
         });
        for (var i in result) {
            $(".grid-container").append(`<input class="playable-item grid-item" type="button" value="` + result[i].display_name + `" file_name="`+  result[i].full_file_name +`"></input>`);
        }
        $(".grid-container")
    }, 'json')
        .fail(function(err) {
            console.log(err);
        });
})