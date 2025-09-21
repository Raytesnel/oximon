import arcade

from pokemon import Monsters
from smash_stage.SmashWorld import SmashWorld
from smash_stage.attacks import SimpleMelee, ALL_ATTACKS
from smash_stage.fighter import Attacks
from utils import load_state


class BattleSplashView(arcade.View):
    def __init__(self, overworld_view, wild_pokemon):
        super().__init__()
        self.wild_pokemon = wild_pokemon
        self.game_view = overworld_view
        self.selected_index = 0
        self.camera = arcade.Camera2D()
        self.character_stats = load_state()
        self.selected_monster = None
        self.enemy_labels = [
            arcade.Text("Enemy:", 550, 400, arcade.color.WHITE, 16),
            arcade.Text(f"Name: {wild_pokemon.name}", 550, 370, arcade.color.WHITE, 14),
            arcade.Text("HP: 100", 550, 350, arcade.color.WHITE, 14),
            arcade.Text("ATK: 20", 550, 330, arcade.color.WHITE, 14),
        ]

    def on_draw(self):
        self.clear()
        self.camera.use()
        for label in self.enemy_labels:
            label.draw()

        for i, (monster_name, monster_data) in enumerate(
            self.character_stats["pokemons"].items()
        ):
            sprite = arcade.Sprite(
                [
                    monster.over_world_sprites.banner
                    for monster in Monsters
                    if monster.name.lower() == monster_name.lower()
                ][0]
            )
            y = 400 - i * 70
            color = (
                arcade.color.YELLOW if i == self.selected_index else arcade.color.WHITE
            )
            if color == arcade.color.YELLOW:
                self.selected_monster = (monster_name, monster_data)
            sprite.center_x = 40
            sprite.center_y = y + 10
            arcade.draw_sprite(sprite)
            text = arcade.Text(
                f"{monster_name} - HP: {monster_data['stats']['health']}",
                80,
                y,
                color,
                14,
            )
            text.draw()

    def on_key_press(self, key, modifiers):
        if key == arcade.key.UP:
            self.selected_index = (self.selected_index - 1) % len(
                self.character_stats["pokemons"]
            )
        elif key == arcade.key.DOWN:
            self.selected_index = (self.selected_index + 1) % len(
                self.character_stats["pokemons"]
            )
        elif key == arcade.key.SPACE:
            wild_monster = next(
                pokemon
                for pokemon in Monsters
                if pokemon.name == self.wild_pokemon.name
            )
            selected_fighter = next(
                monster
                for monster in Monsters
                if monster.name.lower() == self.selected_monster[0].lower()
            ).fighter
            wild_monster_fighter = wild_monster.fighter
            wild_monster_fighter.set_attacks(
                Attacks(
                    up=None,
                    down=None,
                    right=wild_monster.moves[0].move,
                    left=wild_monster.moves[0].move,
                    base=SimpleMelee,
                )
            )
            selected_fighter.set_attacks(
                Attacks(
                    up=ALL_ATTACKS[self.selected_monster[1]["attacks"]["up"]],
                    down=ALL_ATTACKS[self.selected_monster[1]["attacks"]["down"]],
                    left=ALL_ATTACKS[self.selected_monster[1]["attacks"]["left"]],
                    right=ALL_ATTACKS[self.selected_monster[1]["attacks"]["right"]],
                    base=ALL_ATTACKS[self.selected_monster[1]["attacks"]["base"]],
                )
            )
            self.window.show_view(
                SmashWorld(
                    overworld_view=self.game_view,
                    wild_monster=wild_monster_fighter,
                    wild_monster_overworld=self.wild_pokemon,
                    chosen_monster=selected_fighter,
                )
            )
