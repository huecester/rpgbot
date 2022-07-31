local healing = math.random(5, 20);
healing = heal_user(healing);
add_log_entry(user_name .. " ate an apple and healed for " .. healing .. " health.");