Thanks for using the Library README Template!
* Consider the value each optional section brings before removing.
* For guidance, take a look at the commented out examples for each section.
* If the content is lengthy, consider breaking it off into separate pages and linking to it.
* This template is designed with internal libraries in mind. For public repos, remove proprietary information and consider reaching out in [#dev-context](https://shopify.slack.com/archives/CEQ6MR2F7) for assistance.
* Leave feedback about the template in the [Google Doc](https://docs.google.com/document/d/1cE3X_yg5mTBso4d960RiYQL3PEcY6-WCkdw4ik9TpeE/edit?usp=sharing) or in [#dev-context](https://shopify.slack.com/archives/CEQ6MR2F7)

---

# lz4_flex-rb
**Badges (optional):** Services found in ServicesDB have the option to show build badges, offering quick links and build status information. When selecting badges, consider the most important statuses for your repo.\
Can include:
* **Services DB status badge:** Available if your repo is for a service:
  [![lz4-flex-rb](https://services.shopify.io/services/lz4-flex-rb/badge.svg)](https://services.shopify.io/services/lz4-flex-rb).
* **Buildkite status badge:** Available if your repo is for a service. [Follow the help guide to get your badge code](https://buildkite.com/docs/integrations/build-status-badges).
* **CircleCI status badge:** Can be added following [this help guide](https://circleci.com/docs/2.0/status-badges/).
* **Cloudsmith version badge:** Can be found on the badges tab for the package in Cloudsmith.
* **Other badges:** Additional status badges relevant to your project can be found at http://shields.io/.
* **Custom badges:** Custom badges can be added using small image icons of your choice. Popular custom badges include Shipit and Splunk.

<!--
Examples:
* Plus B2B Learning Project: Handshake Importer Prototype - custom badges](https://github.com/Shopify/plus-b2b-learning-project-hs-importer/blob/master/README.md)
-->

[About this library](#about-this-library) | [How to install this library](#how-to-install-this-library) | [Contribute to this library](#contribute-to-this-library-optional) | [Projects & Roadmap](#projects--roadmap-optional) | [Releases](#releases) | [Policies](#policies-optional)

## About this library
**Introduction:** Main goal of the library, and why it's important in 2-3 short paragraphs.

|                |                                                                                                                                      |
|----------------|--------------------------------------------------------------------------------------------------------------------------------------|
| Current status | Current project status (if applicable). Examples include: maintenance, deprecated, beta, etc. Include a link to the roadmap if relevant.                                                 |
| Owner          | Who maintains the library? Link to Vault or Github teams. Break into new section if extensive.                            |
| Help           | Where to go for help or ask questions. Link any relevant help channels, playbooks or resources. Break into new section if extensive. |

<!--
Examples:
* [Ability Client Library - About this repo section](https://github.com/Shopify/ability-client#about-this-repo)
* [Active Merchant Library - Introduction section](https://github.com/activemerchant/active_merchant#active-merchant)
* [Shopify App Library - Introduction paragraph](https://github.com/Shopify/shopify_app#introduction)
* [Business Platform - Stewards table with github teams](https://github.com/Shopify/business-platform/blob/master/README.md#stewards)
* [Seamster - Motivation & intent section](https://github.com/Shopify/seamster/blob/master/README.md#motivation--intent)
-->

## How to install this library
A quick start guide for users who want to install this library. Can include subsections such as:
* Requirements
* Setup
* Library specific steps
* Troubleshooting

<!--
Examples:
* [Ability Client Library - How to use this repo](https://github.com/Shopify/ability-client#how-to-use-this-repo)
* [Shopify App Library - Requirements section](https://github.com/Shopify/shopify_app#requirements)
* [Shopify App Library - Usage section](https://github.com/Shopify/shopify_app#usage)
* [Blaast Library - Detailed initializing section](https://github.com/Shopify/blaast#initializing-from-blast-off)
-->

## How to use this library
A quick start guide for users who want to use the library. Can include subsections such as:
* API spec/link to live API docs
* Examples
* Tutorials
* How to run tests
* Tech Design Docs

If the content exceeds 3 paragraphs or includes large tables/graphs, break out into a new Markdown file, and link to it.

<!--
Examples:
* [Active Merchant Library - GettingStarted.md file and API docs linked in Usage section](https://github.com/activemerchant/active_merchant#usage)
* [Shopify App Library - Usage section](https://github.com/Shopify/shopify_app#usage)
* [Ability Client Library - How to use this repo](https://github.com/Shopify/ability-client#how-to-use-this-repo)

-->

## Contribute to this library (optional)
Details your contribution guidelines, including if the library does not accept them. Can include additional steps such as:
* What is necessary for PRs (formatting, pings)
* Architecture and style guides
* Version release and management (if this information is split into a new file, it should be named RELEASING.md)
  Before removing this section, consider whether a new hire reading this README for the first time would be able to understand how to contribute to the repo. If the content exceeds 3 paragraphs or includes large tables/graphs, break out into a CONTRIBUTING.md file, and link to it.

<!--
Examples:
* [Active Merchant Library - CONTRIBUTING.md file](https://github.com/activemerchant/active_merchant/blob/master/CONTRIBUTING.md)
* [Delivery component - Architecture and style onboarding content](https://github.com/Shopify/shopify/blob/master/components/delivery/README.md#component-architecture-and-style)
* [Oberlo Merchant - Developer onboarding format](https://github.com/Shopify/oberlo-merchant/blob/master/README.md)
* [Business Platform - Development and deployment content](https://github.com/Shopify/business-platform/blob/master/README.md#development)
* [Good CONTRIBUTING.md template gist](https://gist.github.com/PurpleBooth/b24679402957c63ec426)
-->

## Projects & roadmap (optional)
Link to projects and roadmaps where relevant. A high level overview of the future of the library. This section should guide the reader towards understanding what features currently exist and what features are planned.

| Feature name | Feature description                                                         |
|--------------|-----------------------------------------------------------------------------|
| Feature      | Example description of the feature (link to relevant project if applicable). Marked as Done, ongoing or planned. |

<!--
Examples:
* [Ability Client Library - Projects and roadmap section](https://github.com/Shopify/ability-client#projects--roadmap)
* [Magellan - Properties and wishlist content](https://github.com/Shopify/magellan/blob/master/README.md#architecture)
-->

## Releases

This gem is published to [Cloudsmith](https://cloudsmith.io/~shopify/repos/gems/packages).

The procedure to publish a new release version is as follows:

* Update `lib/lz4_flex/rb/version.rb`
* Run bundle install to bump the version of the gem in `Gemfile.lock`
* Open a pull request, review, and merge
* Review commits since the last release to identify user-facing changes that should be included in the release notes
* [Create a release on GitHub](https://github.com/Shopify/lz4-flex-rb/releases/new) with a version number that matches `lib/lz4_flex/rb/version.rb`. More on [creating releases](https://help.github.com/articles/creating-releases).
* [Deploy via Shipit](https://shipit.shopify.io/shopify/lz4-flex-rb/cloudsmith) and see your [latest version on Cloudsmith](https://cloudsmith.io/~shopify/repos/gems/packages/detail/ruby/lz4_flex-rb/latest)

## Policies (optional)
Additional policies that affect contributions or use of this library. This can include topics such as:
* Compatibility
* Stability

<!--
Examples:
* [Active Merchant - stability and compatibility policies](https://github.com/activemerchant/active_merchant#api-stability-policy)
-->
