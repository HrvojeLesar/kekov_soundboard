$('.editModal').on('show.bs.modal', function (event) {
    console.log("kek");
    var button = $(event.relatedTarget) // Button that triggered the modal
    var display_name = button.data('display-name') // Extract info from data-* attributes
    var modal = $(this)
    modal.find('.modal-title').text('Editing ' + display_name)
    modal.find('.modal-body #display_name').val(display_name)
})