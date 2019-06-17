---
layout: post
title: "SemVer: Use It, or Don't!"
subtitle: Stop confusing your users.
---

As a maintainer of a handful of projects that consume libraries, a source of continual frustration for me is the seeming nonchalant disregard for the importance of _consistent_ and _predictable_ versioning, and the wide variety of different [versioning/numbering] schemes that other maintainers use.
In my mind, there are plenty of traditions concerning this matter that already are (and should be) followed, but then a maintainer just does whatever they want and breaks everything in a patch release.
But it doesn't have to be this way.
In this post, I want to share the way that I do versioning, and hopefully convince you that you should use it in your projects&mdash;but, as with all things, pick something and stick to it!

I think part of the problem is that most developers _expect_ their dependencies to follow Semantic Versioning (SemVer).
This is in part because the SemVer specification, when correctly applied, allows maintainers to _predictably_ communicate the nature of the changes that are introduced by a new version of their software.
When new versions are released under SemVer, by design, consumers of those packages don't have to read anything or hear any caveats if their code can upgrade.
