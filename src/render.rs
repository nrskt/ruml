use super::Entity;

pub trait PlantUml {
    fn render(&self) -> String;
    fn render_dependencies(&self, source: Vec<String>) -> String;
}

pub fn render_plantuml(entities: Vec<Entity>) -> String {
    let rendered: Vec<String> = entities
        .clone()
        .into_iter()
        .map(|ent| format!("{}}}", ent.render()))
        .collect();
    let rendered = rendered.join("\n\n");

    let source: Vec<String> = entities
        .clone()
        .into_iter()
        .map(|x| x.name.to_string())
        .collect();

    let dep: Vec<String> = entities
        .clone()
        .into_iter()
        .map(|ent| ent.render_dependencies(source.clone()))
        .collect();
    let dep = dep.join("\n\n");

    format!("@startuml\n\n{}\n{}\n@enduml", rendered, dep)
}
