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
        result.sort((a, b) => { return a.time_stamp - b.time_stamp });
        for (var i in result) {
            $(".grid-container").append(
            `<button class="grid-item" data-toggle="modal" data-target="#editDisplayNameModal-${i}"
                data-full-file-name="${result[i].full_file_name}" data-display-name="${result[i].display_name}" value="${result[i].display_name}"> ${result[i].display_name} <span class="fa fa-pencil-square-o"></span>
                    <a class="item-remove" data-toggle="modal" data-target="#removeModal-${i}"
                    data-full-file-name="${result[i].full_file_name}" data-display-name="${result[i].display_name}">
                        <span class="fa fa-trash-o"></span>
                    </a>
            </button>
    
            <div class="modal fade editModal" id="editDisplayNameModal-${i}" tabindex="-1" role="dialog" aria-labelledby="editDisplayNameModalLabel"
                aria-hidden="true">
                <div class="modal-dialog" role="document">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title"></h5>
                            <button type="button" class="close" data-dismiss="modal" aria-label="Close">
                                <span aria-hidden="true">&times;</span>
                            </button>
                        </div>
                        <form action="./rename" method="POST">
                        <div class="modal-body">
                            <div class="form-group">
                                <label for="display_name" class="col-form-label">Display Name: </label>
                                <input type="hidden" class="form-control" id="full_path" value="${result[i].full_file_name}" name="value">
                                <input type="text" class="form-control" id="display_name" name="new_display_name">
                            </div>
                        </div>
                            <div class="modal-footer">
                                <button type="button" class="btn btn-secondery" data-dismiss="modal">Close</button>
                                <input type="submit" class="btn btn-primary" value="Submit">
                            </div>
                        </form>
                    </div>
                </div>
            </div>
    
            <div class="modal fade removeModal" id="removeModal-${i}" tabindex="-1" role="dialog" aria-labelledby="removeModalLabel"
                aria-hidden="true">
                <div class="modal-dialog" role="document">
                    <div class="modal-content">
                        <div class="modal-header">
                            <h5 class="modal-title">Remove ${result[i].display_name} ?</h5>
                            <button type="button" class="close" data-dismiss="modal" aria-label="Close">
                                <span aria-hidden="true">&times;</span>
                            </button>
                        </div>
                        <form action="./remove" method="POST">
                            <input type="hidden" class="form-control" id="full_path" value="${result[i].full_file_name}" name="value">
                            <div class="modal-footer">
                                <button type="button" class="btn btn-secondery" data-dismiss="modal">Close</button>
                                <input type="submit" class="btn btn-danger" value="Remove">
                            </div>
                        </form>
                    </div>
                </div>
            </div>
            `);
        }
        $(".grid-container")
    }, 'json')
        .fail(function(err) {
            console.log(err);
        });
})