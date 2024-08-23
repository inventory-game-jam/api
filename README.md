# Inventory Game Jam - API

## Endpoints:

### POST /team_score/{team_name}/{score} [Authenticated]
Adds the given score to the total score of the team.

Returns the new team scores.

### POST /player_score/{uuid}/{score} [Authenticated]
Adds the given score to the players score AND the teams total score.

Returns the new team scores.

### PUT /pack/{name} [Authenticated]
Uploads a resourcepack under the given name.

This pack will be accessible under `/packs/{name}`.

Example cURL request for uploading:

```sh
curl -F'file=@myPack.zip' radsteve.net:3000/pack/myEpicPack -X PUT -H "Authorization: Bearer myEpicToken"
```

If this is too much for you, you can also just send @rad the pack and I will add it.

### GET /packs/{name}
Gets a resource pack by name.

### GET / [Authenticated]
Returns all the team scores.

Example output:

```json
[{"name":"team1","total_score":0,"players":[{"uuid":"454c9909-7092-4e6b-bd65-f799099b1ab1","score":0}]}]
```

### PUT /teams/{team_name}/{uuid} [Authenticated]
Adds a player to a team.

Returns the new teams/scores.

### DELETE /teams/{team_name}/{uuid} [Authenticated]
Removes a player and their scores from a team.

Returns the new teams/scores



Note: All endpoints marked with `[Authenticated]` require an `Authorization: Bearer` header. Example: `curl -H "Authorization: Bearer myToken" radsteve.net:3000`
