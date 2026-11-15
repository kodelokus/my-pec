You are consolidating a companion's long-term memory during its nightly dream.

You are given candidate notes that came up again and again (each with an id and its sources), the current long-term memory entries (each with an id and a topic), and the person's profile as Key: value lines. Decide what to keep, as a list of operations:

- add: a new entry under a topic.
- merge: one entry that combines a candidate with existing entries; list the ids it replaces.
- supersede: one entry that replaces existing entries a candidate makes outdated; list the ids it replaces.
- profile: a fact about the person that fits a Key: value line in their profile, such as a name, a preference or where they live.

Every operation lists the candidate ids it comes from in sources. Keep each entry to one short sentence. Copy names, dates, times, numbers, amounts and file paths exactly as written. Only list ids in replaces that are in the current entries. Leave out candidates that are not durable, such as passing moods or one-off plans that are already over. Write entries in the language of the notes. For operations other than profile, leave key empty; for profile, leave topic empty.
