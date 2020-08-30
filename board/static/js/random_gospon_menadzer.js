$(function() {

    function spawn_gospon_menadzer(start) {
        $("body").append(`
            <a href="./volimoZnidarica">
                <img src="./static/favicon.ico" class="gospon-menadzer" style="width: 16px; height: 16px; position: fixed; top:` + start.top + `; left: ` + start.left + `;">
           </a>
        `);
    }

    function fly(end, times_to_spin, scale) {
        let gospon_menadzer = $(".gospon-menadzer");
        gospon_menadzer.animate({deg: 360 * times_to_spin, "top": end.top, "left": end.left}, {
            duration: 5000,
            step: function(now, tween) {
                if (tween.prop === "deg") {
                    gospon_menadzer.css({
                        transform: "rotate(" + now + "deg) scale(" + scale + ")",
                    });
                }
            },
            done: function() { setTimeout(remove_gospon_menadzer, 3000); }
        });
    }

    function remove_gospon_menadzer() {
        $(".gospon-menadzer").remove();
    }

    // inclusive
    function random_between(min, max) {
        return Math.floor(Math.random() * (max - min + 1) + min);
    }

    function animate() {
        let spawn_pos = [];
        let flag = false;
        for(let i = 0; i < 2; i++) {
            spawn_pos[i] = random_between(-100, 200);

            if (Math.floor(Math.random() * 2 ) && flag) {
                // 110 - 200
                spawn_pos[i] =  random_between(110, 200);
            } else if (flag) {
                // -10 - -100
                spawn_pos[i] =  random_between(-100, -10);
            }

            if (spawn_pos[i] > -10 && spawn_pos[i] < 110) {
                flag = true;
            }
        }
        
        spawn_gospon_menadzer({top: spawn_pos[0] + "%", left: spawn_pos[1] + "%"})
        
        let landing_pos = [];
        for (let i = 0; i < 2; i++) {
            landing_pos[i] = random_between(0, 95);
        }

        let times_to_spin = random_between(0, 5);
        let scale = Math.random() * (1 - 0.1 + 1) + 0.1;
        if (random_between(0, 500) >= 490) {
            scale = random_between(10, 30);
        }

        fly({top: landing_pos[0] + "%", left: landing_pos[1] + "%"}, times_to_spin, scale);
    }

    setInterval(function() {
        let num = random_between(0, 100);
        if (num >= 90) {
            animate();
        }
    }, 20000);

});