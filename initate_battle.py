import arcade

from pokemon import Monsters
from smash_stage.SmashWorld import SmashWorld
from smash_stage.fighter import Character
from utils import POKEMON_SPRITES_PATH, load_state, ASSETS_PATH


class Monster:
    def __init__(self, name, hp, atk, image_path, scale):
        self.name = name
        self.hp = hp
        self.atk = atk
        self.image_path = image_path
        self.sprite = arcade.Sprite(image_path, scale=scale)


class BattleSplashView(arcade.View):
    def __init__(self, overworld_view, wild_pokemon):
        super().__init__()
        self.wild_pokemon = wild_pokemon
        self.game_view = overworld_view
        self.selected_index = 0
        self.camera = arcade.Camera2D()
        self.character_stats = load_state()
        self.team_monsters = []
        counter = 0
        for character_monster in self.character_stats["pokemons"]:
            monster_name = list(character_monster.keys())[0]
            self.team_monsters.append(
                Monster(
                    name=monster_name,
                    hp=character_monster[monster_name]["stats"]["health"],
                    atk=character_monster[monster_name]["stats"]["attack"],
                    image_path=POKEMON_SPRITES_PATH
                    / monster_name.title()
                    / "banner.png",
                    scale=1,
                )
            )
            counter += 1
            if counter == 3:
                break

        # Pre-create enemy text labels
        self.enemy_labels = [
            arcade.Text("Enemy:", 550, 400, arcade.color.WHITE, 16),
            arcade.Text(f"Name: {wild_pokemon.name}", 550, 370, arcade.color.WHITE, 14),
            arcade.Text("HP: 100", 550, 350, arcade.color.WHITE, 14),
            arcade.Text("ATK: 20", 550, 330, arcade.color.WHITE, 14),
        ]

    def on_draw(self):
        self.clear()
        self.camera.use()
        # Background
        arcade.draw_lbwh_rectangle_filled(
            bottom=0,
            left=0,
            width=self.window.width,
            height=self.window.height,
            color=arcade.color.BLACK,
        )

        # Draw static enemy info
        for label in self.enemy_labels:
            label.draw()

        # Draw your team with highlight
        for i, monster in enumerate(self.team_monsters):
            y = 400 - i * 70
            color = (
                arcade.color.YELLOW if i == self.selected_index else arcade.color.WHITE
            )

            # Position and draw sprite
            monster.sprite.center_x = 40
            monster.sprite.center_y = y + 10
            arcade.draw_sprite(monster.sprite)

            # Draw name & HP next to sprite
            text = arcade.Text(f"{monster.name} - HP: {monster.hp}", 80, y, color, 14)
            text.draw()

    def on_key_press(self, key, modifiers):
        if key == arcade.key.UP:
            self.selected_index = (self.selected_index - 1) % len(self.team_monsters)
        elif key == arcade.key.DOWN:
            self.selected_index = (self.selected_index + 1) % len(self.team_monsters)
        elif key == arcade.key.SPACE:
            selected = self.team_monsters[
                self.selected_index
            ]  # TODO: emplement attacks and monster from state
            wild_monster_fighter = next(
                pokemon.fighter
                for pokemon in Monsters
                if pokemon.name == self.wild_pokemon.name
            )
            self.window.show_view(
                SmashWorld(
                    self.game_view,
                    wild_monster_fighter,
                    Character(
                        ASSETS_PATH / "sprites" / "pokemon" / "Lightning Mage", "mage"
                    ),  # TODO: move Character object instantion to a the self.team_monsters or so.
                )
            )
