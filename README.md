# Crawl

Crawl is a [little language](https://dl.acm.org/doi/10.1145/6424.315691) to help you run TTRPGs. Use it to define and run procedures:

```
load table "random-encounters.csv"
load table "weather.csv"

procedure day
    roll on table "weather.csv"
    if roll 1-3 on 1d6 => set-fact "party is lost"
    if roll 1-3 on 1d6 => set-fact "day has random encounter"
    if fact? "day has random encounter" => encounter
    reminder "players must consume one day's worth of rations"
end

procedure encounter
	roll on table "random-encounters.csv"
    roll 2d6
        2-4 => set-fact "encounter is hostile"
        5-8 => set-fact "encounter is neutral"
        9-12 => set-fact "encounter is friendly"
    end
end

day
```
