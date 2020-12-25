trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn area_per_perimeter(&self) -> f64 {
        return self.area() / self.perimeter();
    }
}

enum Shapes {
    Circle(Circle),
    Rectangle(Rectangle)
}

trait StaticDispatch<T> {
    fn to_enum(value: T) -> Shapes;
}

fn print_shape(shape: &dyn Shape) {
    println!("area {}, perimeter {}, area_per_perimeter {}", shape.area(), shape.perimeter(), shape.area_per_perimeter());
}

#[derive(Clone)]
struct Circle {
    radius: f64
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius.powi(2)
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

impl StaticDispatch<Circle> for Circle {
    fn to_enum(circle: Circle) -> Shapes {
        Shapes::Circle(circle)
    }
}

#[derive(Clone)]
struct Rectangle {
    width: f64,
    height: f64
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * self.width + 2.0 * self.height
    }
}

impl Rectangle {
    fn to_enum(rectangle: Rectangle) -> Shapes {
        Shapes::Rectangle(rectangle)
    }
}

impl StaticDispatch<Rectangle> for Rectangle {
    fn to_enum(rectangle: Rectangle) -> Shapes {
        Shapes::Rectangle(rectangle)
    }
}

fn main() {
    // create plain objects
    let c1 = Circle { radius: 1.0 };
    let c2 = Circle { radius: 2.0 };
    let r1 = Rectangle { width: 1.0, height: 1.0 };
    let r2 = Rectangle { width: 2.0, height: 3.0 };

    println!("=== Plain ===");
    print_shape(&c1);
    print_shape(&c2);
    print_shape(&r1);
    print_shape(&r2);

    // borrow and store them in a vector
    let mut vec_of_refs: Vec<&dyn Shape> = Vec::new();
    vec_of_refs.push(&c1);
    vec_of_refs.push(&c2);
    vec_of_refs.push(&r1);
    vec_of_refs.push(&r2);

    println!("=== Refs ===");
    for shape in vec_of_refs {
        print_shape(shape);
    }

    // create a vector of boxed objects (i.e. pointers)
    let mut vec_of_boxes: Vec<Box<dyn Shape>> = Vec::new();

    // create boxed objects by allocating new memory and cloning the values (dynamic dispatch)
    vec_of_boxes.push(Box::new(c1.clone()));
    vec_of_boxes.push(Box::new(c2.clone()));
    vec_of_boxes.push(Box::new(r1.clone()));
    vec_of_boxes.push(Box::new(r2.clone()));

    println!("=== Boxes ===");
    for shape in vec_of_boxes {
        print_shape(shape.as_ref());
    }

    // create a vector of enums (i.e. variants)
    let mut vec_of_enums: Vec<Shapes> = Vec::new();

    // move existing objects into enums (static dispatch)
    vec_of_enums.push(Circle::to_enum(c1));
    vec_of_enums.push(Circle::to_enum(c2));
    vec_of_enums.push(Rectangle::to_enum(r1));
    vec_of_enums.push(Rectangle::to_enum(r2));

    println!("=== Enums ===");
    for shape in vec_of_enums {
        match shape {
            Shapes::Circle(circle) => print_shape(&circle),
            Shapes::Rectangle(rectangle) => print_shape(&rectangle)
        }
    }
}
