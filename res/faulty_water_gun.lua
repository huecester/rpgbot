local opponent_damage = math.random(30, 40);
local backfire = math.random() < 0.1;
local self_damage = math.random(50, 60);
if backfire then
	local damage = damage_user(self_damage, 0);
	add_log_entry(user_name .. "'s water gun backfired, dealing " .. damage .. " damage to themselves.");
else
	local damage = damage_opponent(opponent_damage, 0);
	add_log_entry(user_name .. " splashed " .. opponent_name .. " with a water gun, dealing " .. damage .. " damage.");
end