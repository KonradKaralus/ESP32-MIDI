fn main() {

    embed_resource::compile("src/resources.rc", embed_resource::NONE);


    slint_build::compile("ui/ui.slint").unwrap();

}