import arcade


class Attack(arcade.Sprite):
    def __init__(
        self,
        owner,
        textures,
        damage,
        knockback,
        width,
        height,
        direction,
        offset=(0, 0),
        frame_duration=1 / 30,
        lifetime=10,
    ):
        super().__init__()
        self.owner = owner
        self.textures = textures
        self.texture = textures[0]
        self.current_frame = 0
        self.animation_timer = 0
        self.frame_duration = frame_duration
        self.life_timer = lifetime * frame_duration

        self.damage = damage
        self.knockback = knockback
        self.width = width
        self.height = height

        offset_x, offset_y = offset
        self.center_x = owner.center_x + offset_x
        self.center_y = owner.center_y + offset_y

    def update(self, delta_time):
        self.life_timer -= delta_time
        if self.life_timer <= 0:
            self.remove_from_sprite_lists()
            return

        self.animation_timer += delta_time
        if self.animation_timer >= self.frame_duration:
            self.current_frame = (self.current_frame + 1) % len(self.textures)
            self.texture = self.textures[self.current_frame]
            self.animation_timer = 0
