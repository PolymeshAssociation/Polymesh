const React = require('react')
const {Dropdown} = require('semantic-ui-react')
const {InputBond} = require('./InputBond')
const {Balance} = require('oo7-substrate')
const {denominations, denominationInfo} = require('oo7-substrate').denominationInfo

function formatValueNoDenom(n) {
	return `${n.units.toString().replace(/(\d)(?=(\d{3})+$)/g, '$1,')}${n.decimals ? '.' + n.decimals : ''}`
}

function combineValue(v) {
	let d = Math.pow(1000, v.denom)
	let n = v.units
	if (v.decimals) {
		n += v.decimals
		d /= Math.pow(10, v.decimals.length)
	}
	return n * d
}

function defDenom(v, d) {
	if (v.denom == null) {
		v.denom = d
	}
	return v
}

function interpretRender(s) {
	try {
		let m = s.toLowerCase().match(/([0-9,]+)(\.([0-9]*))? *([a-zA-Z]+)?/)
		let di = m[4] ? denominations.indexOf(m[4]) : null
		if (di === -1) {
			return null
		}
		let n = (m[1].replace(',', '').replace(/^0*/, '')) || '0'
		let d = (m[3] || '').replace(/0*$/, '')
		return { denom: di, units: n, decimals: d, origNum: m[1] + (m[2] || ''), origDenom: m[4] || '' }
	}
	catch (e) {
		return null
	}
}

class BalanceBond extends InputBond {
	constructor() {
		super()
	}

	getUnits () {
		return this.state.ok
			? denominations[
				this.state.internal
					? this.state.internal.denom
					: defaultDenom()
			]
			: defaultDenom()
	}

	setUnits (v) {
		let s = this.state.internal
		let d = denominations.indexOf(v)
		s.denom = d
		this.state.internal = s
		this.handleEdit(this.state.display)
	}

	handleBlur () {
		let s = this.state
		if (typeof(s.corrected) === 'string') {
			s.display = s.corrected
			delete s.corrected
			this.setState(s)
		}
	}

	makeAction (p) {
		return p ? 'right' : (<Dropdown
			button
			basic
			floating
			onChange={(_, v) => this.setUnits(v.value)}
			value={this.getUnits()}
			options={denominations
//				.filter(x => x[0] == x[0].toLowerCase())
				.map(d => ({key: d, value: d, text: d}))
			}
		/>)
	}
}

function defaultDenom() {
	return denominations.indexOf(denominationInfo.primary)
}

BalanceBond.defaultProps = {
	placeholder: '0',
	defaultValue: '',
	validator: (u, s) => {
		let q = u === ''
			? {
				denom: s.internal && s.internal.denom !== null
					? s.internal.denom
					: defaultDenom(),
				units: '0',
				decimals: '',
				origNum: '',
				origDenom: ''
			}
			: interpretRender(u, null)
		let d = q && q.denom !== null ? q.origNum : undefined
		if (q) {
			defDenom(q, s.internal ? s.internal.denom : defaultDenom())
		}
		return q ? {
			internal: q,
			display: d,
			corrected: formatValueNoDenom(q),
			external: new Balance(combineValue(q)),
			ok: true
		} : null
	}
}

module.exports = { BalanceBond }
