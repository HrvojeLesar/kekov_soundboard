const progressBar= $("#progressBar");
const uploadForm = document.getElementById("uploadForm");
const MAX_UPLOAD = 10485760;

$("#uploadForm").submit(function(event) {
    event.preventDefault();

    $("p").remove();
    
    const xhr = new XMLHttpRequest();

    xhr.open("POST", "./upload");
    xhr.upload.addEventListener("progress", function(e) {
        let percent = e.lengthComputable ? (e.loaded / e.total) * 100 : 0;
        let percentValue = percent.toFixed(2) + "%";
        progressBar.css("width", percentValue);
        progressBar.text(percentValue);
    });

    xhr.setRequestHeader("Content-Encoding", "multipart/form-data");

    // xhr.onreadystatechange = function() {
    //     xhr.abort();
    //     progressBar.text("Error");
    //     progressBar.removeClass("bg-success");
    //     progressBar.addClass("bg-danger");
    //     $("body").append(`<p style="color: red;">Upload failed</p>`)
    //     console.log("Eto greška");
    // }

    xhr.onload = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 500) {
                xhr.abort();
                progressBar.text("Error");
                progressBar.removeClass("bg-success");
                progressBar.addClass("bg-danger");
                $("body").append(`<p style="color: red;">Upload failed! Please refresh the website</p>`)
                console.log("Eto greška");
            }
        }
    }

    let formData = new FormData(uploadForm);
    let uploadSize = Array.from(formData.entries(), ([key, prop]) => (
        {[key]: {
            "ContentLength":
            typeof prop === "string" ? prop.length : prop.size
        }}));
    
    if (uploadSize[0].inputFile.ContentLength > MAX_UPLOAD) {
        $("body").append(`<p style="color: red;">File size too large</p>`)
    } else {
        $("input").remove();
        xhr.send(formData);
    }
});