const React = require('react');
const {Input} = require('semantic-ui-react');
const {Bond} = require('oo7');
const {ReactiveComponent} = require('oo7-react');

class InputBond extends ReactiveComponent {
	constructor (extraReactiveProps = []) {
		super(extraReactiveProps instanceof Array ? ['defaultValue', ...extraReactiveProps] : ['defaultValue']);
		this.state = {
			display: null,
			internal: null,
			external: null,
			ok: false,
			extra: {},
			onlyDefault: true
		};
		this.editLock = false;
	}

	componentDidMount () {
		if (this.props.bond && this.props.reversible) {
			this.tieKey = this.props.bond.tie(v => {
				this.handleEdit(v);
			});
		}
	}

	componentWillUnmount () {
		if (this.props.bond && this.props.reversible && this.tieKey) {
			this.props.bond.untie(this.tieKey);
		}
		if (this.cleanup) {
			this.cleanup();
			delete this.cleanup;
		}
	}

	handleEdit(v, onlyDefault = false) {
//		console.log('handleEdit', v, onlyDefault);
		if (this.editLock)
			return;

		this.resetDefaultValueUpdate();
		this.latestEdit = Symbol();

		let f = function (b) {
//			console.log('updating...', b);
			if (typeof(b) === 'string') {
				b = { display: b, external: b, internal: b };
			}
			if (typeof(b) !== 'object') {
				throw { message: 'Invalid value returned from validity function. Must be object with internal and optionally external, display, blurred fields or null', b };
			}
//			console.log('ok...', b);
			if (b === null) {
				this.setState({ok: false});
			} else {
				this.setState(s => {
					let i = b && b.hasOwnProperty('internal') ? b.internal : s.internal;
					return {
						ok: true,
						internal: i,
						display: typeof(b.display) === 'string' ? b.display : s.display,
						corrected: b.corrected,
						extra: b.extra || {},
						onlyDefault,
						external: b && b.hasOwnProperty('external') ? b.external : i
					};
				});
			}
			/// Horrible duck-typing, necessary since the specific Bond class instance here is different to the other libraries since it's
			/// pre-webpacked in a separate preprocessing step.
			if (Bond.instanceOf(this.props.bond)) {
				if (b === null) {
					this.props.bond.reset();
				} else {
					this.editLock = true;
					this.props.bond.changed(b && b.hasOwnProperty('external') ? b.external : b && b.hasOwnProperty('internal') ? b.internal : this.state.internal);
					this.editLock = false;
				}
			}
		}.bind(this);

		if (this.cleanup) {
			this.cleanup();
			delete this.cleanup;
		}
		this.setState({display: v, onlyDefault});

		if (typeof(this.props.validator) !== 'function') {
			f(v);
		} else {
			let a = v !== undefined && this.props.validator(v, this.state);
			if (a instanceof Promise || Bond.instanceOf(a)) {
				let thisSymbol = this.latestEdit;
				let m = a.tie(r => {
					if (this.latestEdit === thisSymbol) {
						f(r);
					}
				});
				this.cleanup = () => a.untie(m);
			} else {
				f(a);
			}
		}
	}

	handleBlur () {
		this.setState(s => typeof(s.corrected) === 'string' && typeof(s.display) === 'string'
			? { display: s.corrected, corrected: undefined }
			: {}
		);
	}

	resetDefaultValueUpdate () {
		if (this.lastDefaultValueUpdate) {
//			console.log('kill update');
			window.clearTimeout(this.lastDefaultValueUpdate);
			delete this.lastDefaultValueUpdate;
		}
	}

	resetValueToDefault () {
		this.resetDefaultValueUpdate();
//		console.log('schedule update');
		this.lastDefaultValueUpdate = window.setTimeout(() => { this.handleEdit(this.state.defaultValue, true); }, 0);
	}

	render () {
		if (this.state.onlyDefault && typeof(this.state.defaultValue) === 'string' && this.state.display !== this.state.defaultValue) {
//			console.log('newDefault', this.state.defaultValue);
			this.resetValueToDefault();
		}
		let icon = this.makeIcon ? this.makeIcon() : this.props.icon
		let iconPosition = this.makeIcon ? this.makeIcon(true) : this.props.iconPosition
		return (<Input
			className={this.props.className}
			style={this.props.style}
			name={this.props.name}
			children={this.props.children}
			disabled={this.props.disabled}
			fluid={this.props.fluid}
			placeholder={this.props.placeholder}
			inverted={this.props.inverted}
			loading={this.props.loading}
			size={this.props.size}
			transparent={this.props.transparent}
			type='text'
			value={this.state.display != null ? this.state.display : this.state.defaultValue != null ? this.state.defaultValue : ''}
			error={!this.state.ok}
			onKeyDown={this.props.onKeyDown}
			onChange={(e, v) => this.handleEdit(v.value)}
			onBlur={() => this.handleBlur()}
			action={this.makeAction ? this.makeAction() : this.props.action}
			label={this.makeLabel ? this.makeLabel() : this.props.label}
			labelPosition={this.makeLabel ? this.makeLabel(true) : this.props.labelPosition}
			icon={icon}
			iconPosition={iconPosition}
		/>);
	}
}
InputBond.defaultProps = {
	placeholder: '',
	defaultValue: '',
	reversible: false
};

module.exports = { InputBond };
