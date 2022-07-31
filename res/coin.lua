local heal = math.random() < 0.5;
local health = math.random(20, 35);
if heal then
	local healing = heal_opponent(health);
	add_log_entry(user_name .. " flipped " .. healing .. " healing against " .. opponent_name .. ".");
else
	local damage = damage_opponent(health, 0);
	add_log_entry(user_name .. " flipped " .. damage .. " damage against " .. opponent_name .. ".");
end