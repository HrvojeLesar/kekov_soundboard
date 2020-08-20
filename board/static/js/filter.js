$(function () {
    $(".filter").keyup(function() {
        let input = $(".filter").val().toLowerCase();
        let items = $(".grid-item");
        
        for (let i = 0; i < items.length; i++) {
            if (items[i].value.toLowerCase().indexOf(input) != -1) {
                items[i].style.display = "";
            } else {
                items[i].style.display = "none";
            }
        }
    });
});