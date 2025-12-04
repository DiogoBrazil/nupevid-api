-- Create ENUM types for victim fields

CREATE TYPE violence_type_enum AS ENUM ('Physical', 'Sexual', 'Psychological', 'Moral');

CREATE TYPE has_children_enum AS ENUM ('Yes', 'No', 'Pregnant');
