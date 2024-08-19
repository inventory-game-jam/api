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
curl -F'file=@myPack.zip' $API_URL/pack/myEpicPack -X PUT -H "Authorization: Bearer myEpicToken"
```

### GET /packs/{name}
Gets a resourece pack by name.

### GET / [Authenticated]
Returns all the team scores.

Example output:

```json
[{"name":"team1","total_score":0,"players":[{"uuid":"454c9909-7092-4e6b-bd65-f799099b1ab1","score":0}]}]
```

### Team management endpoints planned.
