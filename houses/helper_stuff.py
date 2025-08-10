def check_object_collision(player, obj):
    if not obj:
        return False
    (left, top), (right, _), (_, _), (_, bottom) = obj.shape
    return (
        player.right > left and
        player.left < right and
        player.top > bottom and
        player.bottom < top
    )

