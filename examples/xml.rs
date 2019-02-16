extern crate azul;

const TEST_XML: &str = "

<component name='start-screen'>
    <div id='start_screen'>
        <div id='last_projects_column'>
             <p id='last_projects_header'>LAST PROJECTS</p>
             <div id='project_btn_container'>
                <p id='new_project_btn' onleftmouseup='menu_new_project'>+</p>
                <p id='open_project_btn' onleftmouseup='menu_open_project'>Open project</p>
             </div>
        </div>
        <div id='map_preview_container'>
            <div id='map_preview' />
            <div id='map_details_view' />
        </div>
    </div>
</component>

<component name='toolbar' fn='render_calendar'>
    <div class='hello'><p>selectedDate</p></div>
</component>

<app>
    <div id='start_screen_wrapper'>
        <start-screen />
    </div>
    <calendar
        selectedDate='01.01.2018'
        minimumDate='01.01.1970'
        maximumDate='31.12.2034'
        firstDayOfWeek='sunday'
        gridVisible='false'
        dateSelectable='true'
        horizontalHeaderFormat='Mon'
        verticalHeaderFormat='S'
        navigationBarVisible='true'
    />
    <form id='test_form'>
        <section id='my_test_section'>
            <textinput placeholder='Type here...' />
        </section>
    </form>
</app>
";

fn main() {
    let xml = azul::xml::parse_xml_string(TEST_XML).unwrap();
    println!("xml: {:#?}", xml);
}