$(function () {
    $(".filter").attr("placeholder", "Click or press 'S' to search");

    document.addEventListener("keydown", function(ev) {
        if (document.activeElement.className === "filter") {
            if (ev.code === "Escape" || ev.code === "Enter") {
                $(".filter").blur();
            }
            return;
        }

        if (ev.code === "KeyS") {
            ev.preventDefault();
            $(".filter").focus();
        }
    });

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