table! {
    items (id) {
        id -> Int4,
        name -> Varchar,
        description -> Text,
        icon -> Bpchar,
        lua -> Text,
    }
}

table! {
    weapons (id) {
        id -> Int4,
        name -> Varchar,
        icon -> Bpchar,
        damage_range -> Nullable<Int4range>,
        crit_ratio -> Nullable<Float8>,
        crit_multiplier -> Nullable<Int4>,
        pierce -> Nullable<Int4>,
    }
}

allow_tables_to_appear_in_same_query!(
    items,
    weapons,
);
