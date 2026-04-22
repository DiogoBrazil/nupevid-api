-- Create ENUM types for offender fields

CREATE TYPE security_force_enum AS ENUM (
    'Military Police',
    'Civil Police',
    'Penal Police',
    'Fire Department',
    'Federal Highway Police',
    'Federal Police',
    'Municipal Guard'
);

CREATE TYPE relationship_to_victim_enum AS ENUM (
    'Spouse',
    'Ex-Spouse',
    'Mother',
    'Father',
    'Stepfather',
    'Stepmother',
    'Son/Daughter',
    'Sibling',
    'Cousin',
    'Grandfather',
    'Grandmother',
    'Brother/Sister-in-law',
    'Father/Mother-in-law',
    'Son-in-law',
    'Daughter-in-law',
    'Uncle/Aunt'
);
