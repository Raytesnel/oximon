import arcade

from smash_stage.SmashWorld import SmashWorld


class BattleSplashView(arcade.View):
    def __init__(self, player_sprite_path, banner_path, game_view, wild_pokemon):
        super().__init__()
        self.wild_pokemon = wild_pokemon
        self.timer = 0
        self.show_duration = 1.5
        self.game_view = game_view
        banner_widht = 555
        scaling = self.window.width / banner_widht
        self.banner = arcade.Sprite(banner_path,scale=scaling)
        self.player_sprite = arcade.Sprite(player_sprite_path,scale=0.5)
        self.enemy_sprite = arcade.Sprite(self.wild_pokemon.sprite_file_location, scale=0.2)
        self.sprites = arcade.SpriteList()
        self.on_show()
        self.sprites.append(self.banner)
        self.sprites.append(self.player_sprite)
        self.sprites.append(self.enemy_sprite)

    def on_show(self):
        self.banner.center_x = self.game_view.player.center_x
        self.banner.center_y = self.game_view.player.center_y
        self.player_sprite.center_x = self.game_view.player.center_x - self.window.width //2 + self.player_sprite.width//2
        self.player_sprite.center_y = self.game_view.player.center_y
        self.enemy_sprite.center_x = self.game_view.player.center_x + self.window.width //2 - self.enemy_sprite.width//2
        self.enemy_sprite.center_y = self.game_view.player.center_y

    def on_draw(self):
        self.clear()
        self.sprites.draw()

    def on_update(self, delta_time):
        self.timer += delta_time
        if self.timer > self.show_duration:
            self.window.show_view(SmashWorld(self.game_view))
            # self.window.show_view(self.game_view)


    # def on_key_press(self, key, modifiers):
    #     self.window.show_view(self.game_view)
